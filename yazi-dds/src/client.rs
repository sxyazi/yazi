use std::{collections::{HashMap, HashSet}, mem, str::FromStr};

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, select, sync::mpsc, task::JoinHandle, time};
use yazi_shared::RoCell;

use crate::{body::{Body, BodyBye, BodyHi}, ClientReader, ClientWriter, Payload, Pubsub, Server, Stream};

pub(super) static ID: RoCell<u64> = RoCell::new();
pub(super) static PEERS: RoCell<RwLock<HashMap<u64, Peer>>> = RoCell::new();

pub(super) static QUEUE_TX: RoCell<mpsc::UnboundedSender<String>> = RoCell::new();
pub(super) static QUEUE_RX: RoCell<mpsc::UnboundedReceiver<String>> = RoCell::new();

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
	pub(super) fn serve() {
		let mut rx = QUEUE_RX.drop();
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

						if line.is_empty() {
							continue;
						} else if line.starts_with("hey,") {
							Self::handle_hey(line);
						} else {
							Payload::from_str(&line).map(|p| p.emit()).ok();
						}
					}
				}
			}
		});
	}

	pub async fn shot(kind: &str, receiver: u64, severity: Option<u16>, body: &str) -> Result<()> {
		Body::validate(kind)?;

		let sender = severity.map(Into::into).unwrap_or(*ID);
		let payload = format!(
			"{}\n{kind},{receiver},{sender},{body}\n{}\n",
			Payload::new(BodyHi::borrowed(Default::default())),
			Payload::new(BodyBye::borrowed())
		);

		let (mut lines, mut writer) = Stream::connect().await?;
		writer.write_all(payload.as_bytes()).await?;
		writer.flush().await?;
		drop(writer);

		while let Ok(Some(s)) = lines.next_line().await {
			if matches!(s.split(',').next(), Some(kind) if kind == "bye") {
				break;
			}
		}

		Ok(())
	}

	#[inline]
	pub(super) fn push<'a>(payload: impl Into<Payload<'a>>) {
		QUEUE_TX.send(format!("{}\n", payload.into())).ok();
	}

	#[inline]
	pub(super) fn able(&self, ability: &str) -> bool { self.abilities.contains(ability) }

	async fn connect(server: &mut Option<JoinHandle<()>>) -> (ClientReader, ClientWriter) {
		let mut first = true;
		loop {
			if let Ok(conn) = Stream::connect().await {
				Pubsub::pub_from_hi();
				return conn;
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

	async fn reconnect(server: &mut Option<JoinHandle<()>>) -> (ClientReader, ClientWriter) {
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
