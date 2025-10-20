use std::{io, path::Path};

use yazi_config::vfs::{ProviderSftp, Vfs};
use yazi_fs::provider::{Attrs, FileBuilder};
use yazi_sftp::fs::Flags;
use yazi_shared::scheme::SchemeRef;

pub struct Gate {
	sftp: super::Sftp,

	append:     bool,
	attrs:      Attrs,
	create:     bool,
	create_new: bool,
	read:       bool,
	truncate:   bool,
	write:      bool,
}

impl FileBuilder for Gate {
	type File = yazi_sftp::fs::File;

	fn append(&mut self, append: bool) -> &mut Self {
		self.append = append;
		self
	}

	fn attrs(&mut self, attrs: Attrs) -> &mut Self {
		self.attrs = attrs;
		self
	}

	fn create(&mut self, create: bool) -> &mut Self {
		self.create = create;
		self
	}

	fn create_new(&mut self, create_new: bool) -> &mut Self {
		self.create_new = create_new;
		self
	}

	async fn new(scheme: SchemeRef<'_>) -> io::Result<Self> {
		let sftp: super::Sftp = match scheme {
			SchemeRef::Sftp(name) => Vfs::provider::<&ProviderSftp>(name).await?.into(),
			_ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Not an SFTP URL"))?,
		};

		Ok(Self {
			sftp,

			append: false,
			attrs: Attrs::default(),
			create: false,
			create_new: false,
			read: false,
			truncate: false,
			write: false,
		})
	}

	async fn open<P>(&self, path: P) -> io::Result<Self::File>
	where
		P: AsRef<Path>,
	{
		let mut flags = Flags::empty();
		if self.append {
			flags |= Flags::APPEND;
		}
		if self.create {
			flags |= Flags::CREATE;
		}
		if self.create_new {
			flags |= Flags::CREATE | Flags::EXCLUDE;
		}
		if self.read {
			flags |= Flags::READ;
		}
		if self.truncate {
			flags |= Flags::TRUNCATE;
		}
		if self.write {
			flags |= Flags::WRITE;
		}

		let attrs = super::Attrs(self.attrs).into();

		Ok(self.sftp.op().await?.open(&path, flags, &attrs).await?)
	}

	fn read(&mut self, read: bool) -> &mut Self {
		self.read = read;
		self
	}

	fn truncate(&mut self, truncate: bool) -> &mut Self {
		self.truncate = truncate;
		self
	}

	fn write(&mut self, write: bool) -> &mut Self {
		self.write = write;
		self
	}
}
