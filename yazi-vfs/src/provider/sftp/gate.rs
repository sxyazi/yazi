use std::io;

use yazi_config::vfs::{ServiceSftp, Vfs};
use yazi_fs::provider::{Attrs, FileBuilder};
use yazi_sftp::fs::Flags;
use yazi_shared::url::{AsUrl, Url};

use crate::provider::sftp::Conn;

#[derive(Clone, Copy, Default)]
pub struct Gate(crate::provider::Gate);

impl From<Gate> for Flags {
	fn from(Gate(g): Gate) -> Self {
		let mut flags = Self::empty();
		if g.append {
			flags |= Self::APPEND;
		}
		if g.create {
			flags |= Self::CREATE;
		}
		if g.create_new {
			flags |= Self::CREATE | Self::EXCLUDE;
		}
		if g.read {
			flags |= Self::READ;
		}
		if g.truncate {
			flags |= Self::TRUNCATE;
		}
		if g.write {
			flags |= Self::WRITE;
		}
		flags
	}
}

impl FileBuilder for Gate {
	type File = yazi_sftp::fs::File;

	fn append(&mut self, append: bool) -> &mut Self {
		self.0.append(append);
		self
	}

	fn attrs(&mut self, attrs: Attrs) -> &mut Self {
		self.0.attrs(attrs);
		self
	}

	fn create(&mut self, create: bool) -> &mut Self {
		self.0.create(create);
		self
	}

	fn create_new(&mut self, create_new: bool) -> &mut Self {
		self.0.create_new(create_new);
		self
	}

	async fn open<U>(&self, url: U) -> io::Result<Self::File>
	where
		U: AsUrl,
	{
		let url = url.as_url();
		let (path, (name, config)) = match url {
			Url::Sftp { loc, domain } => (*loc, Vfs::service::<&ServiceSftp>(domain).await?),
			_ => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Not an SFTP URL: {url:?}")))?,
		};

		let conn = Conn { name, config }.roll().await?;
		let flags = Flags::from(*self);
		let attrs = super::Attrs(self.0.attrs).try_into().unwrap_or_default();

		let result = conn.open(path, flags, &attrs).await;
		if self.0.create_new
			&& let Err(yazi_sftp::Error::Status(status)) = &result
			&& status.is_failure()
			&& conn.lstat(path).await.is_ok()
		{
			return Err(io::Error::from(io::ErrorKind::AlreadyExists));
		}

		Ok(result?)
	}

	fn read(&mut self, read: bool) -> &mut Self {
		self.0.read(read);
		self
	}

	fn truncate(&mut self, truncate: bool) -> &mut Self {
		self.0.truncate(truncate);
		self
	}

	fn write(&mut self, write: bool) -> &mut Self {
		self.0.write(write);
		self
	}
}
