use std::io;

use yazi_fs::engine::{Attrs, Engine, FileBuilder};
use yazi_sftp::fs::Flags;
use yazi_shared::url::AsUrl;

use crate::engine::sftp::Sftp;

#[derive(Clone, Copy, Default)]
pub struct Demand(yazi_fs::engine::Demand);

impl From<Demand> for Flags {
	fn from(Demand(demand): Demand) -> Self {
		let mut flags = Self::empty();
		if demand.append {
			flags |= Self::APPEND;
		}
		if demand.create {
			flags |= Self::CREATE;
		}
		if demand.create_new {
			flags |= Self::CREATE | Self::EXCLUDE;
		}
		if demand.read {
			flags |= Self::READ;
		}
		if demand.truncate {
			flags |= Self::TRUNCATE;
		}
		if demand.write {
			flags |= Self::WRITE;
		}
		flags
	}
}

impl FileBuilder for Demand {
	type File = yazi_sftp::fs::File;

	fn append(&mut self, append: bool) -> &mut Self {
		self.0.append = append;
		self
	}

	fn attrs(&mut self, attrs: Attrs) -> &mut Self {
		self.0.attrs = attrs;
		self
	}

	fn create(&mut self, create: bool) -> &mut Self {
		self.0.create = create;
		self
	}

	fn create_new(&mut self, create_new: bool) -> &mut Self {
		self.0.create_new = create_new;
		self
	}

	async fn open<U>(&self, url: U) -> io::Result<Self::File>
	where
		U: AsUrl,
	{
		let engine = Sftp::new(url.as_url()).await?;
		let conn = engine.op().await?;

		let flags = Flags::from(*self);
		let attrs = super::Attrs(self.0.attrs).try_into().unwrap_or_default();
		let result = conn.open(engine.path, flags, &attrs).await;

		if self.0.create_new
			&& let Err(yazi_sftp::Error::Status(status)) = &result
			&& status.is_failure()
			&& conn.lstat(engine.path).await.is_ok()
		{
			return Err(io::Error::from(io::ErrorKind::AlreadyExists));
		}

		Ok(result?)
	}

	fn read(&mut self, read: bool) -> &mut Self {
		self.0.read = read;
		self
	}

	fn truncate(&mut self, truncate: bool) -> &mut Self {
		self.0.truncate = truncate;
		self
	}

	fn write(&mut self, write: bool) -> &mut Self {
		self.0.write = write;
		self
	}
}
