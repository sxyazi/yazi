use std::{any::TypeId, collections::HashMap, io::{self, ErrorKind}, sync::Arc, time::Duration};

use parking_lot::Mutex;
use russh::{ChannelStream, client::Msg};
use serde::Serialize;
use tokio::{io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf}, select, sync::{mpsc, oneshot}};

use crate::{Error, Id, Packet, Receiver, responses};

pub struct Session {
	tx:                    mpsc::UnboundedSender<Vec<u8>>,
	id:                    Id,
	pub(super) callback:   Mutex<HashMap<u32, oneshot::Sender<Packet<'static>>>>,
	pub(super) extensions: Mutex<HashMap<String, String>>,
}

impl Drop for Session {
	fn drop(&mut self) { self.tx.send(vec![]).ok(); }
}

impl Session {
	pub(super) fn make(stream: ChannelStream<Msg>) -> Arc<Self> {
		let (tx, mut rx) = mpsc::unbounded_channel();
		let me = Arc::new(Self {
			tx,
			id: Id::default(),
			callback: Default::default(),
			extensions: Default::default(),
		});

		async fn read(reader: &mut ReadHalf<ChannelStream<Msg>>) -> io::Result<Vec<u8>> {
			let len = reader.read_u32().await?;
			let mut buf = vec![0; len as usize];
			reader.read_exact(&mut buf).await?;
			Ok(buf)
		}

		async fn write(writer: &mut WriteHalf<ChannelStream<Msg>>, buf: Vec<u8>) -> io::Result<()> {
			if buf.is_empty() {
				Err(io::Error::from(ErrorKind::BrokenPipe))
			} else {
				writer.write_all(&buf).await
			}
		}

		let me_ = me.clone();
		let (mut reader, mut writer) = tokio::io::split(stream);
		tokio::spawn(async move {
			while let Some(data) = rx.recv().await {
				if let Err(e) = write(&mut writer, data).await
					&& e.kind() == ErrorKind::BrokenPipe
				{
					rx.close();
					writer.shutdown().await.ok();
					for (id, cb) in me_.callback.lock().drain() {
						cb.send(responses::Status::connection_lost(id).into()).ok();
					}
					break;
				}
			}
		});

		let me_ = me.clone();
		tokio::spawn(async move {
			loop {
				select! {
					result = read(&mut reader) => {
						let buf = match result {
							Ok(b) => b,
							Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
								me_.tx.send(vec![]).ok();
								break;
							},
							Err(_) => continue,
						};

						if let Ok(packet) = crate::from_bytes(&buf)
							&& let Some(cb) = me_.callback.lock().remove(&packet.id())
						{
							cb.send(packet).ok();
						}
					}
					_ = me_.tx.closed() => break,
				}
			}
		});

		me
	}

	pub async fn send<'a, I, O>(self: &Arc<Self>, input: I) -> Result<O, Error>
	where
		I: Into<Packet<'a>> + Serialize,
		O: TryFrom<Packet<'static>, Error = Error> + 'static,
	{
		self.send_with_timeout(input, Duration::from_secs(45)).await
	}

	pub fn send_sync<'a, I>(self: &Arc<Self>, input: I) -> Result<Receiver, Error>
	where
		I: Into<Packet<'a>> + Serialize,
	{
		let mut request: Packet = input.into();
		if request.id() == 0 {
			request = request.with_id(self.id.next());
		}

		let id = request.id();
		let (tx, rx) = oneshot::channel();

		self.callback.lock().insert(id, tx);
		self.tx.send(crate::to_bytes(request)?)?;
		Ok(Receiver::new(self, id, rx))
	}

	pub async fn send_with_timeout<'a, I, O>(
		self: &Arc<Self>,
		input: I,
		timeout: Duration,
	) -> Result<O, Error>
	where
		I: Into<Packet<'a>> + Serialize,
		O: TryFrom<Packet<'static>, Error = Error> + 'static,
	{
		match tokio::time::timeout(timeout, self.send_sync(input)?).await?? {
			Packet::Status(status) if TypeId::of::<O>() != TypeId::of::<responses::Status>() => {
				Err(Error::Status(status))
			}
			response => response.try_into(),
		}
	}

	pub fn is_closed(self: &Arc<Self>) -> bool { self.tx.is_closed() }
}
