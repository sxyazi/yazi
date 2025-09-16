use std::{io, path::Path};

use yazi_sftp::fs::{Attrs, Flags};

use crate::provider::FileBuilder;

#[derive(Default)]
pub struct Gate {
	append:     bool,
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

	fn create(&mut self, create: bool) -> &mut Self {
		self.create = create;
		self
	}

	fn create_new(&mut self, create_new: bool) -> &mut Self {
		self.create_new = create_new;
		self
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
		Ok(super::Sftp::op().await?.open(&path, flags, Attrs::default()).await?)
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
