use std::{mem, str::FromStr};

use anyhow::Result;
use hashbrown::{HashMap, HashSet};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, select, sync::mpsc, task::JoinHandle, time};
use tracing::error;
use yazi_macro::try_format;
use yazi_shared::{Id, RoCell};

use crate::{ClientReader, ClientWriter, Payload, Pubsub, Server, Stream, ember::{Ember, EmberHey}};

pub static ID: RoCell<Id> = RoCell::new();
pub(super) static PEERS: RoCell<RwLock<HashMap<Id, Peer>>> = RoCell::new();

pub(super) static QUEUE_TX: RoCell<mpsc::UnboundedSender<String>> = RoCell::new();
pub(super) static QUEUE_RX: RoCell<mpsc::UnboundedReceiver<String>> = RoCell::new();

#[derive(Debug)]
pub struct Client {
	pub(super) id:        Id,
	pub(super) tx:        mpsc::UnboundedSender<String>,
	pub(super) abilities: HashSet<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Peer {
	pub(super) abilities: HashSet<String>,
}

impl Client {
	/// Connect to an existing server or start a new one.
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
							continue;  // Heartbeat, ignore
						}

						let payload = match Payload::from_str(&line) {
							Ok(p) => p,
							Err(e) => {
								error!("Failed to parse DDS payload:\n{line}\n\nError:\n{e}");
								continue;
							}
						};

						if let Ember::Hey(hey) = &payload.body {
							Self::handle_hey(hey);
						}

						payload.try_flush().ok();
						payload.emit();
					}
				}
			}
		});
	}

	pub(super) fn push<'a>(payload: impl Into<Payload<'a>>) -> Result<()> {
		Ok(QUEUE_TX.send(try_format!("{}\n", payload.into())?)?)
	}

	pub(super) fn able(&self, ability: &str) -> bool { self.abilities.contains(ability) }

	async fn connect(server: &mut Option<JoinHandle<()>>) -> (ClientReader, ClientWriter) {
		let mut first = true;
		loop {
			if let Ok(conn) = Stream::connect().await {
				Pubsub::pub_inner_hi();
				tracing::debug!("Connected to existing DDS server on instance {ID}");
				return conn;
			}

			server.take().map(|h| h.abort());
			match Server::make().await {
				Ok(h) => {
					*server = Some(h);
					super::STATE.load_or_create().await;
					tracing::debug!("Started new DDS server on instance {ID}");
				}
				Err(e) => {
					tracing::error!("Could not connect to or start a new DDS server on instance {ID}: {e}");
				}
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

	fn handle_hey(body: &EmberHey) {
		*PEERS.write() = body
			.peers
			.iter()
			.filter(|&(id, _)| *id != *ID)
			.map(|(&id, peer)| (id, peer.clone()))
			.collect();
	}
}

impl Peer {
	pub(super) fn new(abilities: &HashSet<String>) -> Self { Self { abilities: abilities.clone() } }

	pub fn able(&self, ability: &str) -> bool { self.abilities.contains(ability) }
}
