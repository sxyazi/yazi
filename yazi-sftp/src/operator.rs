use std::{ops::Deref, sync::Arc};

use russh::{ChannelStream, client::Msg};
use typed_path::UnixPathBuf;

use crate::{AsSftpPath, Error, Receiver, Session, SftpPath, fs::{Attrs, File, Flags, ReadDir}, requests, responses};

pub struct Operator(Arc<Session>);

impl Deref for Operator {
	type Target = Arc<Session>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<&Arc<Session>> for Operator {
	fn from(session: &Arc<Session>) -> Self { Self(session.clone()) }
}

impl Operator {
	pub fn make(stream: ChannelStream<Msg>) -> Self { Self(Session::make(stream)) }

	pub async fn init(&mut self) -> Result<(), Error> {
		let version: responses::Version = self.send(requests::Init::default()).await?;
		*self.extensions.lock() = version.extensions;
		Ok(())
	}

	pub async fn open<'a, P>(&self, path: P, flags: Flags, attrs: &'a Attrs) -> Result<File, Error>
	where
		P: AsSftpPath<'a>,
	{
		let handle: responses::Handle = self.send(requests::Open::new(path, flags, attrs)).await?;

		Ok(File::new(&self.0, handle.handle))
	}

	pub fn close(&self, handle: &str) -> Result<Receiver, Error> {
		self.send_sync(requests::Close::new(handle))
	}

	pub fn read(&self, handle: &str, offset: u64, len: u32) -> Result<Receiver, Error> {
		self.send_sync(requests::Read::new(handle, offset, len))
	}

	pub fn write(&self, handle: &str, offset: u64, data: &[u8]) -> Result<Receiver, Error> {
		self.send_sync(requests::Write::new(handle, offset, data))
	}

	pub async fn lstat<'a, P>(&self, path: P) -> Result<Attrs, Error>
	where
		P: AsSftpPath<'a>,
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
		P: AsSftpPath<'a>,
	{
		let status: responses::Status = self.send(requests::SetStat::new(path, attrs)).await?;
		status.into()
	}

	pub async fn fsetstat(&self, handle: &str, attrs: &Attrs) -> Result<(), Error> {
		let status: responses::Status = self.send(requests::FSetStat::new(handle, attrs)).await?;
		status.into()
	}

	pub async fn read_dir<'a, P>(&'a self, dir: P) -> Result<ReadDir, Error>
	where
		P: AsSftpPath<'a>,
	{
		let dir: SftpPath = dir.as_sftp_path();
		let handle: responses::Handle = self.send(requests::OpenDir::new(&dir)).await?;

		Ok(ReadDir::new(&self.0, dir, handle.handle))
	}

	pub async fn remove<'a, P>(&self, path: P) -> Result<(), Error>
	where
		P: AsSftpPath<'a>,
	{
		let status: responses::Status = self.send(requests::Remove::new(path)).await?;
		status.into()
	}

	pub async fn mkdir<'a, P>(&self, path: P, attrs: Attrs) -> Result<(), Error>
	where
		P: AsSftpPath<'a>,
	{
		let status: responses::Status = self.send(requests::Mkdir::new(path, attrs)).await?;
		status.into()
	}

	pub async fn rmdir<'a, P>(&self, path: P) -> Result<(), Error>
	where
		P: AsSftpPath<'a>,
	{
		let status: responses::Status = self.send(requests::Rmdir::new(path)).await?;
		status.into()
	}

	pub async fn realpath<'a, P>(&self, path: P) -> Result<UnixPathBuf, Error>
	where
		P: AsSftpPath<'a>,
	{
		let mut name: responses::Name = self.send(requests::Realpath::new(path)).await?;
		if name.items.is_empty() {
			Err(Error::custom("realpath returned no names"))
		} else {
			Ok(name.items.swap_remove(0).name.into_owned().into())
		}
	}

	pub async fn stat<'a, P>(&self, path: P) -> Result<Attrs, Error>
	where
		P: AsSftpPath<'a>,
	{
		let attrs: responses::Attrs = self.send(requests::Stat::new(path)).await?;
		Ok(attrs.attrs)
	}

	pub async fn rename<'a, F, T>(&self, from: F, to: T) -> Result<(), Error>
	where
		F: AsSftpPath<'a>,
		T: AsSftpPath<'a>,
	{
		let status: responses::Status = self.send(requests::Rename::new(from, to)).await?;
		status.into()
	}

	pub async fn rename_posix<'a, F, T>(&self, from: F, to: T) -> Result<(), Error>
	where
		F: AsSftpPath<'a>,
		T: AsSftpPath<'a>,
	{
		if self.extensions.lock().get("posix-rename@openssh.com").is_none_or(|s| s != "1") {
			return Err(Error::Unsupported);
		}

		let data = requests::ExtendedRename::new(from, to);
		let status: responses::Status =
			self.send(requests::Extended::new("posix-rename@openssh.com", data)).await?;
		status.into()
	}

	pub async fn readlink<'a, P>(&self, path: P) -> Result<UnixPathBuf, Error>
	where
		P: AsSftpPath<'a>,
	{
		let mut name: responses::Name = self.send(requests::Readlink::new(path)).await?;
		if name.items.is_empty() {
			Err(Error::custom("readlink returned no names"))
		} else {
			Ok(name.items.swap_remove(0).name.into_owned().into())
		}
	}

	pub async fn symlink<'a, L, O>(&self, original: O, link: L) -> Result<(), Error>
	where
		O: AsSftpPath<'a>,
		L: AsSftpPath<'a>,
	{
		let status: responses::Status = self.send(requests::Symlink::new(original, link)).await?;
		status.into()
	}

	pub fn fsync(&self, handle: &str) -> Result<Receiver, Error> {
		if self.extensions.lock().get("fsync@openssh.com").is_none_or(|s| s != "1") {
			return Err(Error::Unsupported);
		}

		let data = requests::ExtendedFsync::new(handle);
		self.send_sync(requests::Extended::new("fsync@openssh.com", data))
	}

	pub async fn hardlink<'a, O, L>(&self, original: O, link: L) -> Result<(), Error>
	where
		O: AsSftpPath<'a>,
		L: AsSftpPath<'a>,
	{
		if self.extensions.lock().get("hardlink@openssh.com").is_none_or(|s| s != "1") {
			return Err(Error::Unsupported);
		}

		let data = requests::ExtendedHardlink::new(original, link);
		let status: responses::Status =
			self.send(requests::Extended::new("hardlink@openssh.com", data)).await?;
		status.into()
	}

	pub async fn limits(&self) -> Result<responses::ExtendedLimits, Error> {
		if self.extensions.lock().get("limits@openssh.com").is_none_or(|s| s != "1") {
			return Err(Error::Unsupported);
		}

		let extended: responses::Extended =
			self.send(requests::Extended::new("limits@openssh.com", requests::ExtendedLimits)).await?;
		extended.try_into()
	}
}
