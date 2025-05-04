use std::{collections::{HashMap, HashSet}, mem, str::FromStr};

use anyhow::{Context, Result, bail};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, select, sync::mpsc, task::JoinHandle, time};
use tracing::error;
use yazi_shared::{Id, RoCell};

use crate::{ClientReader, ClientWriter, Payload, Pubsub, Server, Stream, body::{Body, BodyBye, BodyHi}};

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

#[derive(Debug, Serialize, Deserialize)]
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
							continue;
						} else if line.starts_with("hey,") {
							Self::handle_hey(&line);
						} else if let Err(e) = Payload::from_str(&line).map(|p| p.emit()) {
							error!("Could not parse payload:\n{line}\n\nError:\n{e}");
						}
					}
				}
			}
		});
	}

	/// Connect to an existing server to send a single message.
	pub async fn shot(kind: &str, receiver: Id, body: &str) -> Result<()> {
		Body::validate(kind)?;

		let payload = format!(
			"{}\n{kind},{receiver},{ID},{body}\n{}\n",
			Payload::new(BodyHi::borrowed(Default::default())),
			Payload::new(BodyBye::owned())
		);

		let (mut lines, mut writer) = Stream::connect().await?;
		writer.write_all(payload.as_bytes()).await?;
		writer.flush().await?;
		drop(writer);

		let (mut peers, mut version) = Default::default();
		while let Ok(Some(line)) = lines.next_line().await {
			match line.split(',').next() {
				Some("hey") => {
					if let Ok(Body::Hey(hey)) = Payload::from_str(&line).map(|p| p.body) {
						(peers, version) = (hey.peers, Some(hey.version));
					}
				}
				Some("bye") => break,
				_ => {}
			}
		}

		if version.as_deref() != Some(BodyHi::version()) {
			bail!(
				"Incompatible version (Ya {}, Yazi {}). Restart all `ya` and `yazi` processes if you upgrade either one.",
				BodyHi::version(),
				version.as_deref().unwrap_or("Unknown")
			);
		}

		match (receiver, peers.get(&receiver).map(|p| p.able(kind))) {
			// Send to all receivers
			(Id(0), _) if peers.is_empty() => {
				bail!("No receiver found. Check if any receivers are running.")
			}
			(Id(0), _) if peers.values().all(|p| !p.able(kind)) => {
				bail!("No receiver has the ability to receive `{kind}` messages.")
			}
			(Id(0), _) => {}

			// Send to a specific receiver
			(_, Some(true)) => {}
			(_, Some(false)) => {
				bail!("Receiver `{receiver}` does not have the ability to receive `{kind}` messages.")
			}
			(_, None) => bail!("Receiver `{receiver}` not found. Check if the receiver is running."),
		}

		Ok(())
	}

	/// Connect to an existing server and listen in on the messages that are being
	/// sent by other yazi instances:
	///   - If no server is running, fail right away;
	///   - If a server is closed, attempt to reconnect forever.
	pub async fn draw(kinds: HashSet<&str>) -> Result<()> {
		async fn make(kinds: &HashSet<&str>) -> Result<ClientReader> {
			let (lines, mut writer) = Stream::connect().await?;
			let hi = Payload::new(BodyHi::borrowed(kinds.clone()));
			writer.write_all(format!("{hi}\n").as_bytes()).await?;
			writer.flush().await?;
			Ok(lines)
		}

		let mut lines = make(&kinds).await.context("No running Yazi instance found")?;
		loop {
			match lines.next_line().await? {
				Some(s) => {
					let kind = s.split(',').next();
					if matches!(kind, Some(kind) if kinds.contains(kind)) {
						println!("{s}");
					}
				}
				None => loop {
					time::sleep(time::Duration::from_secs(1)).await;
					if let Ok(new) = make(&kinds).await {
						lines = new;
						break;
					}
				},
			}
		}
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

	fn handle_hey(s: &str) {
		if let Ok(Body::Hey(mut hey)) = Payload::from_str(s).map(|p| p.body) {
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
