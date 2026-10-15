use std::sync::Arc;

use crate::{Error, Operator, Receiver, Session, fs::Attrs, requests};

pub struct FileHandle {
	op:     Operator,
	handle: String,
}

impl Drop for FileHandle {
	fn drop(&mut self) { self.op.close(&self.handle).ok(); }
}

impl FileHandle {
	pub(super) fn new(session: &Arc<Session>, handle: impl Into<String>) -> Self {
		Self { op: Operator::from(session), handle: handle.into() }
	}

	pub async fn fstat(&self) -> Result<Attrs, Error> { self.op.fstat(&self.handle).await }

	pub async fn fsetstat(&self, attrs: &Attrs) -> Result<(), Error> {
		self.op.fsetstat(&self.handle, attrs).await
	}

	pub(super) fn read(&self, offset: u64, len: u32) -> Result<Receiver, Error> {
		self.op.read(&self.handle, offset, len)
	}

	pub(super) fn write(&self, offset: u64, data: &[u8]) -> Result<Receiver, Error> {
		self.op.write(&self.handle, offset, data)
	}

	pub(super) fn fsync(&self) -> Result<Receiver, Error> { self.op.fsync(&self.handle) }

	pub(super) fn fstat_sync(&self) -> Result<Receiver, Error> {
		self.op.send_sync(requests::Fstat::new(&self.handle))
	}
}
