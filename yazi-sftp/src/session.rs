use std::{any::TypeId, collections::HashMap, io::ErrorKind, path::PathBuf, sync::Arc};

use parking_lot::Mutex;
use russh::{ChannelStream, client::Msg};
use serde::Serialize;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, select, sync::{mpsc, oneshot}};

use crate::{ByteStr, Error, Id, Packet, fs::{Attrs, File, Flags, ReadDir}, requests, responses};

pub struct Session {
	tx:            mpsc::UnboundedSender<Vec<u8>>,
	pub(crate) id: Id,
	callback:      Arc<Mutex<HashMap<u32, oneshot::Sender<Packet<'static>>>>>,
	extensions:    HashMap<String, String>,
}

impl Session {
	pub fn make(stream: ChannelStream<Msg>) -> Self {
		let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
		let (mut reader, mut writer) = tokio::io::split(stream);
		let callback = Arc::new(Mutex::new(HashMap::<_, oneshot::Sender<_>>::new()));

		tokio::spawn(async move {
			while let Some(data) = rx.recv().await {
				if data.is_empty() {
					rx.close();
					writer.shutdown().await.ok();
					break;
				} else {
					writer.write_all(&data).await.ok();
				}
			}
		});

		let (tx_, callback_) = (tx.clone(), callback.clone());
		tokio::spawn(async move {
			loop {
				select! {
					Ok(len) = reader.read_u32() => {
						let mut buf = vec![0; len as usize];
						if let Err(e) = reader.read_exact(&mut buf).await && e.kind() == ErrorKind::UnexpectedEof {
							tx_.send(vec![]).ok();
							break;
						}
						if let Ok(packet) = crate::from_bytes(&buf)
							&& let Some(cb) = callback_.lock().remove(&packet.id())
						{
							cb.send(packet).ok();
						}
					}
					_ = tx_.closed() => break,
				}
			}
		});

		Self { tx, id: Id::default(), callback, extensions: HashMap::new() }
	}

	pub async fn init(&mut self) -> Result<(), Error> {
		let version: responses::Version = self.send(requests::Init::default()).await?;
		self.extensions = version.extensions;
		Ok(())
	}

	pub async fn open<'a, P>(&self, path: P, flags: Flags, attrs: Attrs) -> Result<File<'_>, Error>
	where
		P: Into<ByteStr<'a>>,
	{
		let handle: responses::Handle = self.send(requests::Open::new(path, flags, attrs)).await?;

		Ok(File::new(self, handle.handle))
	}

	pub fn close(&self, handle: &str) -> Result<oneshot::Receiver<Packet<'static>>, Error> {
		self.send_sync(requests::Close::new(handle))
	}

	pub fn read(
		&self,
		handle: &str,
		offset: u64,
		len: u32,
	) -> Result<oneshot::Receiver<Packet<'static>>, Error> {
		self.send_sync(requests::Read::new(handle, offset, len))
	}

	pub fn write(
		&self,
		handle: &str,
		offset: u64,
		data: &[u8],
	) -> Result<oneshot::Receiver<Packet<'static>>, Error> {
		self.send_sync(requests::Write::new(handle, offset, data))
	}

	pub async fn lstat<'a, P>(&self, path: P) -> Result<Attrs, Error>
	where
		P: Into<ByteStr<'a>>,
	{
		let attrs: responses::Attrs = self.send(requests::Lstat::new(path)).await?;
		Ok(attrs.attrs)
	}

	pub async fn fstat(&self, handle: &str) -> Result<Attrs, Error> {
		let attrs: responses::Attrs = self.send(requests::Fstat::new(handle)).await?;
		Ok(attrs.attrs)
	}

	pub async fn setstat<'a, P>(&self, path: P, attrs: Attrs) -> Result<(), Error>
	where
		P: Into<ByteStr<'a>>,
	{
		let status: responses::Status = self.send(requests::SetStat::new(path, attrs)).await?;
		status.into()
	}

	pub async fn fsetstat(&self, handle: &str, attrs: Attrs) -> Result<(), Error> {
		let status: responses::Status = self.send(requests::FSetStat::new(handle, attrs)).await?;
		status.into()
	}

	pub async fn read_dir<'a>(&'a self, dir: impl Into<ByteStr<'a>>) -> Result<ReadDir<'a>, Error> {
		let dir: ByteStr = dir.into();
		let handle: responses::Handle = self.send(requests::OpenDir::new(&dir)).await?;

		Ok(ReadDir::new(self, dir, handle.handle))
	}

	pub async fn remove<'a, P>(&self, path: P) -> Result<(), Error>
	where
		P: Into<ByteStr<'a>>,
	{
		let status: responses::Status = self.send(requests::Remove::new(path)).await?;
		status.into()
	}

	pub async fn mkdir<'a, P>(&self, path: P, attrs: Attrs) -> Result<(), Error>
	where
		P: Into<ByteStr<'a>>,
	{
		let status: responses::Status = self.send(requests::Mkdir::new(path, attrs)).await?;
		status.into()
	}

	pub async fn rmdir<'a, P>(&self, path: P) -> Result<(), Error>
	where
		P: Into<ByteStr<'a>>,
	{
		let status: responses::Status = self.send(requests::Rmdir::new(path)).await?;
		status.into()
	}

	pub async fn realpath<'a, P>(&self, path: P) -> Result<PathBuf, Error>
	where
		P: Into<ByteStr<'a>>,
	{
		let mut name: responses::Name = self.send(requests::Realpath::new(path)).await?;
		if name.items.is_empty() {
			Err(Error::custom("realpath returned no names"))
		} else {
			Ok(name.items.swap_remove(0).name.into_path())
		}
	}

	pub async fn stat<'a, P>(&self, path: P) -> Result<Attrs, Error>
	where
		P: Into<ByteStr<'a>>,
	{
		let attrs: responses::Attrs = self.send(requests::Stat::new(path)).await?;
		Ok(attrs.attrs)
	}

	pub async fn rename<'a, F, T>(&self, from: F, to: T) -> Result<(), Error>
	where
		F: Into<ByteStr<'a>>,
		T: Into<ByteStr<'a>>,
	{
		let status: responses::Status = self.send(requests::Rename::new(from, to)).await?;
		status.into()
	}

	pub async fn readlink<'a, P>(&self, path: P) -> Result<PathBuf, Error>
	where
		P: Into<ByteStr<'a>>,
	{
		let mut name: responses::Name = self.send(requests::Readlink::new(path)).await?;
		if name.items.is_empty() {
			Err(Error::custom("readlink returned no names"))
		} else {
			Ok(name.items.swap_remove(0).name.into_path())
		}
	}

	pub async fn symlink<'a, L, O>(&self, link: L, original: O) -> Result<(), Error>
	where
		L: Into<ByteStr<'a>>,
		O: Into<ByteStr<'a>>,
	{
		let status: responses::Status = self.send(requests::Symlink::new(link, original)).await?;
		status.into()
	}

	pub fn fsync(&self, handle: &str) -> Result<oneshot::Receiver<Packet<'static>>, Error> {
		if self.extensions.get("fsync@openssh.com").is_none_or(|s| s != "1") {
			return Err(Error::Unsupported);
		}

		let data = requests::ExtendedFsync::new(handle);
		self.send_sync(requests::Extended::new("fsync@openssh.com", data))
	}

	pub async fn hardlink<'a, O, L>(&self, original: O, link: L) -> Result<(), Error>
	where
		O: Into<ByteStr<'a>>,
		L: Into<ByteStr<'a>>,
	{
		if self.extensions.get("hardlink@openssh.com").is_none_or(|s| s != "1") {
			return Err(Error::Unsupported);
		}

		let data = requests::ExtendedHardlink::new(original, link);
		let status: responses::Status =
			self.send(requests::Extended::new("hardlink@openssh.com", data)).await?;
		status.into()
	}

	pub async fn limits(&self) -> Result<responses::ExtendedLimits, Error> {
		if self.extensions.get("limits@openssh.com").is_none_or(|s| s != "1") {
			return Err(Error::Unsupported);
		}

		let extended: responses::Extended =
			self.send(requests::Extended::new("limits@openssh.com", requests::ExtendedLimits)).await?;
		extended.try_into()
	}

	pub async fn send<'a, I, O>(&self, input: I) -> Result<O, Error>
	where
		I: Into<Packet<'a>> + Serialize,
		O: TryFrom<Packet<'static>, Error = Error> + 'static,
	{
		match self.send_sync(input)?.await? {
			Packet::Status(status) if TypeId::of::<O>() != TypeId::of::<responses::Status>() => {
				Err(Error::Status(status))
			}
			response => response.try_into(),
		}
	}

	pub fn send_sync<'a, I>(&self, input: I) -> Result<oneshot::Receiver<Packet<'static>>, Error>
	where
		I: Into<Packet<'a>> + Serialize,
	{
		let mut request: Packet = input.into();
		if request.id() == 0 {
			request = request.with_id(self.id.next());
		}

		let (tx, rx) = oneshot::channel();
		self.callback.lock().insert(request.id(), tx);
		self.tx.send(crate::to_bytes(request)?)?;

		Ok(rx)
	}
}
