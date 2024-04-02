use std::{collections::{HashMap, HashSet}, mem, str::FromStr};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines, ReadHalf, WriteHalf}, select, sync::mpsc, task::JoinHandle, time};
use yazi_shared::RoCell;

use crate::{body::Body, Payload, Pubsub, Server};

pub(super) static ID: RoCell<u64> = RoCell::new();
pub(super) static PEERS: RoCell<RwLock<HashMap<u64, Peer>>> = RoCell::new();
pub(super) static QUEUE: RoCell<mpsc::UnboundedSender<String>> = RoCell::new();

#[cfg(not(unix))]
use tokio::net::TcpStream;
#[cfg(unix)]
use tokio::net::UnixStream;

#[derive(Debug)]
pub struct Client {
	pub(super) id:        u64,
	pub(super) tx:        mpsc::UnboundedSender<String>,
	pub(super) abilities: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Peer {
	pub(super) abilities: HashSet<String>,
}

impl Client {
	pub(super) fn serve(mut rx: mpsc::UnboundedReceiver<String>) {
		while rx.try_recv().is_ok() {}

		tokio::spawn(async move {
			let mut server = None;
			let (mut lines, mut writer) = Self::connect(&mut server).await;

			loop {
				select! {
					Some(payload) = rx.recv() => {
						if writer.write_all(payload.as_bytes()).await.is_err() {
							(lines, writer) = Self::reconnect(&mut server).await;
							writer.write_all(payload.as_bytes()).await.ok(); // Retry once
						}
					}
					Ok(next) = lines.next_line() => {
						let Some(line) = next else {
							(lines, writer) = Self::reconnect(&mut server).await;
							continue;
						};

						if line.starts_with("hey,") {
							Self::handle_hey(line);
						} else {
							Payload::from_str(&line).map(|p| p.emit()).ok();
						}
					}
				}
			}
		});
	}

	#[inline]
	pub(super) fn push<'a>(payload: impl Into<Payload<'a>>) {
		QUEUE.send(format!("{}\n", payload.into())).ok();
	}

	#[inline]
	pub(super) fn able(&self, ability: &str) -> bool { self.abilities.contains(ability) }

	#[cfg(unix)]
	async fn connect(
		server: &mut Option<JoinHandle<()>>,
	) -> (Lines<BufReader<ReadHalf<UnixStream>>>, WriteHalf<UnixStream>) {
		let mut first = true;
		loop {
			if let Ok(stream) = UnixStream::connect("/tmp/yazi.sock").await {
				Pubsub::pub_from_hi();
				let (reader, writer) = tokio::io::split(stream);
				return (BufReader::new(reader).lines(), writer);
			}

			server.take().map(|h| h.abort());
			*server = Server::make().await.ok();
			if server.is_some() {
				super::STATE.load_or_create().await;
			}

			if mem::replace(&mut first, false) && server.is_some() {
				continue;
			}
			time::sleep(time::Duration::from_secs(1)).await;
		}
	}

	#[cfg(not(unix))]
	async fn connect(
		server: &mut Option<JoinHandle<()>>,
	) -> (Lines<BufReader<ReadHalf<TcpStream>>>, WriteHalf<TcpStream>) {
		let mut first = true;
		loop {
			if let Ok(stream) = TcpStream::connect("127.0.0.1:33581").await {
				Pubsub::pub_from_hi();
				let (reader, writer) = tokio::io::split(stream);
				return (BufReader::new(reader).lines(), writer);
			}

			server.take().map(|h| h.abort());
			*server = Server::make().await.ok();
			if mem::replace(&mut first, false) && server.is_some() {
				continue;
			}

			time::sleep(time::Duration::from_secs(1)).await;
		}
	}

	#[cfg(unix)]
	async fn reconnect(
		server: &mut Option<JoinHandle<()>>,
	) -> (Lines<BufReader<ReadHalf<UnixStream>>>, WriteHalf<UnixStream>) {
		PEERS.write().clear();

		time::sleep(time::Duration::from_millis(500)).await;
		Self::connect(server).await
	}

	#[cfg(not(unix))]
	async fn reconnect(
		server: &mut Option<JoinHandle<()>>,
	) -> (Lines<BufReader<ReadHalf<TcpStream>>>, WriteHalf<TcpStream>) {
		PEERS.write().clear();

		time::sleep(time::Duration::from_millis(500)).await;
		Self::connect(server).await
	}

	fn handle_hey(s: String) {
		if let Ok(Body::Hey(mut hey)) = Payload::from_str(&s).map(|p| p.body) {
			hey.peers.retain(|&id, _| id != *ID);
			*PEERS.write() = hey.peers;
		}
	}
}

impl Peer {
	#[inline]
	pub(super) fn new(abilities: &HashSet<String>) -> Self { Self { abilities: abilities.clone() } }

	#[inline]
	pub(super) fn able(&self, ability: &str) -> bool { self.abilities.contains(ability) }
}
