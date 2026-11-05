use std::{mem, str::FromStr};

use anyhow::Result;
use foldhash::fast::FixedState;
use hashbrown::{HashMap, HashSet};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, select, sync::mpsc, task::JoinHandle, time};
use yazi_boot::BOOT;
use yazi_macro::{error, try_format};
use yazi_shared::id::Id;
use yazi_shim::{CompactString, cell::RoCell};

use crate::{ClientReader, ClientWriter, Payload, Pubsub, Server, Stream, ember::{Ember, EmberHey}};

pub(super) static PEERS: RwLock<HashMap<Id, Peer, FixedState>> =
	RwLock::new(HashMap::with_hasher(FixedState::with_seed(0)));

pub(super) static QUEUE_TX: RoCell<mpsc::UnboundedSender<String>> = RoCell::new();

#[derive(Debug)]
pub struct Client {
	pub(super) id:        Id,
	pub(super) tx:        mpsc::UnboundedSender<String>,
	pub(super) abilities: HashSet<CompactString>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Peer {
	abilities: HashSet<CompactString>,
}

impl Client {
	/// Connect to an existing server or start a new one.
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
					next = lines.next_line() => {
						let Ok(Some(line)) = next else {
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
				yazi_macro::debug!("Connected to existing DDS server on instance {}", BOOT.id);
				return conn;
			}

			server.take().map(|h| h.abort());
			match Server::make().await {
				Ok(h) => {
					*server = Some(h);
					super::STATE.load_or_create().await;
					yazi_macro::debug!("Started new DDS server on instance {}", BOOT.id);
				}
				Err(e) => {
					yazi_macro::error!(
						"Could not connect to or start a new DDS server on instance {}: {e}",
						BOOT.id
					);
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
			.filter(|&(id, _)| *id != BOOT.id)
			.map(|(&id, peer)| (id, peer.clone()))
			.collect();
	}
}

impl Peer {
	pub(super) fn new(abilities: &HashSet<CompactString>) -> Self {
		Self { abilities: abilities.clone() }
	}

	pub fn able(&self, ability: &str) -> bool { self.abilities.contains(ability) }
}
