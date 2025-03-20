use std::{collections::HashMap, str::FromStr, time::Duration};

use anyhow::Result;
use parking_lot::RwLock;
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, select, sync::mpsc::{self, UnboundedReceiver}, task::JoinHandle, time};
use yazi_shared::{Id, RoCell};

use crate::{Client, ClientWriter, Payload, Peer, STATE, Stream, body::{Body, BodyBye, BodyHey}};

pub(super) static CLIENTS: RoCell<RwLock<HashMap<Id, Client>>> = RoCell::new();

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
									Self::handle_bye(id, rx, writer).await;
									break;
								}

								let mut parts = line.splitn(4, ',');
								let Some(kind) = parts.next() else { continue };
								let Some(receiver) = parts.next().and_then(|s| s.parse().ok()) else { continue };
								let Some(sender) = parts.next().and_then(|s| s.parse::<u64>().ok()) else { continue };

								let clients = CLIENTS.read();
								let clients: Vec<_> = if receiver == 0 {
									clients.values().filter(|c| c.able(kind)).collect()
								} else if let Some(c) = clients.get(&receiver).filter(|c| c.able(kind)) {
									vec![c]
								} else {
									vec![]
								};

								if clients.is_empty() {
									continue;
								}

								if receiver == 0 && kind.starts_with('@') {
									let Some(body) = parts.next() else { continue };
									if !STATE.set(kind, sender, body) { continue }
								}

								line.push('\n');
								clients.into_iter().for_each(|c| _ = c.tx.send(line.clone()));
							}
							else => break
						}
					}

					let mut clients = CLIENTS.write();
					if id.and_then(|id| clients.remove(&id)).is_some() {
						Self::handle_hey(&clients);
					}
				});
			}
		}))
	}

	fn handle_hi(s: String, id: &mut Option<Id>, tx: mpsc::UnboundedSender<String>) {
		let Ok(payload) = Payload::from_str(&s) else { return };
		let Body::Hi(hi) = payload.body else { return };

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

	fn handle_hey(clients: &HashMap<Id, Client>) {
		let payload = format!(
			"{}\n",
			Payload::new(BodyHey::owned(
				clients.values().map(|c| (c.id, Peer::new(&c.abilities))).collect()
			))
		);
		clients.values().for_each(|c| _ = c.tx.send(payload.clone()));
	}

	async fn handle_bye(id: Id, mut rx: UnboundedReceiver<String>, mut writer: ClientWriter) {
		while let Ok(payload) = rx.try_recv() {
			if writer.write_all(payload.as_bytes()).await.is_err() {
				break;
			}
		}

		_ = writer
			.write_all(BodyBye::owned().with_receiver(id).with_sender(Id(0)).to_string().as_bytes())
			.await;

		writer.flush().await.ok();
	}
}
