use std::{collections::HashMap, str::FromStr, time::Duration};

use anyhow::Result;
use parking_lot::RwLock;
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, select, sync::mpsc, task::JoinHandle, time};
use yazi_shared::RoCell;

use crate::{body::{Body, BodyBye, BodyHey}, Client, Payload, Peer, Stream, STATE};

pub(super) static CLIENTS: RoCell<RwLock<HashMap<u64, Client>>> = RoCell::new();

pub(super) struct Server;

impl Server {
	pub(super) async fn make() -> Result<JoinHandle<()>> {
		CLIENTS.write().clear();
		let listener = Stream::bind().await?;

		Ok(tokio::spawn(async move {
			while let Ok((stream, _)) = listener.accept().await {
				let (tx, mut rx) = mpsc::unbounded_channel::<String>();
				let (reader, mut writer) = tokio::io::split(stream);

				tokio::spawn(async move {
					let mut id = None;
					let mut lines = BufReader::new(reader).lines();
					loop {
						select! {
							Some(payload) = rx.recv() => {
								if writer.write_all(payload.as_bytes()).await.is_err() {
									break;
								}
							}
							_ = time::sleep(Duration::from_secs(5)) => {
								if writer.write_u8(b'\n').await.is_err() {
									break;
								}
							}
							Ok(Some(mut line)) = lines.next_line() => {
								if line.starts_with("hi,") {
									Self::handle_hi(line, &mut id, tx.clone());
									continue;
								}

								let Some(id) = id else { continue };
								if line.starts_with("bye,") {
									writer.write_all(BodyBye::borrowed().with_receiver(id).with_sender(0).to_string().as_bytes()).await.ok();
									break;
								}

								let mut parts = line.splitn(4, ',');
								let Some(kind) = parts.next() else { continue };
								let Some(receiver) = parts.next().and_then(|s| s.parse().ok()) else { continue };
								let Some(sender) = parts.next().and_then(|s| s.parse::<u64>().ok()) else { continue };

								let clients = CLIENTS.read();
								let clients: Vec<_> = if receiver == 0 {
									clients.values().filter(|c| c.id != id && c.able(kind)).collect()
								} else if let Some(c) = clients.get(&receiver).filter(|c| c.id != id && c.able(kind)) {
									vec![c]
								} else {
									vec![]
								};

								if clients.is_empty() {
									continue;
								}

								if receiver == 0 && sender <= u16::MAX as u64 {
									let Some(body) = parts.next() else { continue };
									if !STATE.set(kind, sender as u16, body) { continue }
								}

								line.push('\n');
								clients.into_iter().for_each(|c| _ = c.tx.send(line.clone()));
							}
							else => break
						}
					}
					Self::handle_bye(id);
				});
			}
		}))
	}

	fn handle_hi(s: String, id: &mut Option<u64>, tx: mpsc::UnboundedSender<String>) {
		let Ok(payload) = Payload::from_str(&s) else { return };
		let Body::Hi(hi) = payload.body else { return };

		if payload.sender <= u16::MAX as u64 {
			return; // The kind of static messages cannot be "hi"
		}

		if id.is_none() {
			if let Some(ref state) = *STATE.read() {
				state.values().for_each(|s| _ = tx.send(s.clone()));
			}
		}

		let mut clients = CLIENTS.write();
		id.replace(payload.sender).and_then(|id| clients.remove(&id));
		clients.insert(payload.sender, Client {
			id: payload.sender,
			tx,
			abilities: hi.abilities.into_iter().map(|s| s.into_owned()).collect(),
		});

		Self::handle_hey(&clients);
	}

	fn handle_hey(clients: &HashMap<u64, Client>) {
		let payload = format!(
			"{}\n",
			Payload::new(
				BodyHey { peers: clients.values().map(|c| (c.id, Peer::new(&c.abilities))).collect() }
					.into()
			)
		);
		clients.values().for_each(|c| _ = c.tx.send(payload.clone()));
	}

	fn handle_bye(id: Option<u64>) {
		let mut clients = CLIENTS.write();
		if id.and_then(|id| clients.remove(&id)).is_some() {
			Self::handle_hey(&clients);
		}
	}
}
