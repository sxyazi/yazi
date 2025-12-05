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

		let flags = Flags::from(*self);
		let attrs = super::Attrs(self.0.attrs).into();
		Ok(Conn { name, config }.roll().await?.open(path, flags, &attrs).await?)
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
