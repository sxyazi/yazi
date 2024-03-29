use std::{collections::HashMap, str::FromStr, time::Duration};

use anyhow::Result;
use parking_lot::RwLock;
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, select, sync::mpsc, task::JoinHandle, time};
use yazi_shared::RoCell;

use crate::{body::{Body, BodyHey}, Client, Payload, Peer, STATE};

pub(super) static CLIENTS: RoCell<RwLock<HashMap<u64, Client>>> = RoCell::new();

pub(super) struct Server;

impl Server {
	pub(super) async fn make() -> Result<JoinHandle<()>> {
		CLIENTS.write().clear();
		let listener = Self::bind().await?;

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

								let mut parts = line.splitn(4, ',');
								let Some(id) = id else { continue };
								let Some(kind) = parts.next() else { continue };
								let Some(receiver) = parts.next().and_then(|s| s.parse().ok()) else { continue };
								let Some(severity) = parts.next().and_then(|s| s.parse::<u8>().ok()) else { continue };

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

								if receiver == 0 && severity > 0 {
									let Some(body) = parts.next() else { continue };
									STATE.add(format!("{}_{severity}_{kind}", Body::tab(kind, body)), &line);
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

	#[cfg(unix)]
	#[inline]
	async fn bind() -> Result<tokio::net::UnixListener> {
		tokio::fs::remove_file("/tmp/yazi.sock").await.ok();
		Ok(tokio::net::UnixListener::bind("/tmp/yazi.sock")?)
	}

	#[cfg(not(unix))]
	#[inline]
	async fn bind() -> Result<tokio::net::TcpListener> {
		Ok(tokio::net::TcpListener::bind("127.0.0.1:33581").await?)
	}

	fn handle_hi(s: String, id: &mut Option<u64>, tx: mpsc::UnboundedSender<String>) {
		let Ok(Body::Hi(hi)) = Payload::from_str(&s).map(|p| p.body) else { return };

		let mut clients = CLIENTS.write();
		id.replace(hi.id).and_then(|id| clients.remove(&id));

		if let Some(ref state) = *STATE.read() {
			state.values().for_each(|s| _ = tx.send(format!("{s}\n")));
		}

		clients.insert(hi.id, Client { id: hi.id, tx, abilities: hi.abilities });
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
