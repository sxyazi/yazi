use std::{io, path::Path};

use yazi_shared::scheme::SchemeRef;

use crate::{cha::Cha, provider::FileBuilder};

pub enum Gate {
	Local(super::local::Gate),
	Sftp(super::sftp::Gate),
}

impl From<super::local::Gate> for Gate {
	fn from(value: super::local::Gate) -> Self { Self::Local(value) }
}

impl From<super::sftp::Gate> for Gate {
	fn from(value: super::sftp::Gate) -> Self { Self::Sftp(value) }
}

impl FileBuilder for Gate {
	type File = super::RwFile;

	fn append(&mut self, append: bool) -> &mut Self {
		match self {
			Self::Local(g) => _ = g.append(append),
			Self::Sftp(g) => _ = g.append(append),
		};
		self
	}

	fn cha(&mut self, cha: Cha) -> &mut Self {
		match self {
			Self::Local(g) => _ = g.cha(cha),
			Self::Sftp(g) => _ = g.cha(cha),
		};
		self
	}

	fn create(&mut self, create: bool) -> &mut Self {
		match self {
			Self::Local(g) => _ = g.create(create),
			Self::Sftp(g) => _ = g.create(create),
		};
		self
	}

	fn create_new(&mut self, create_new: bool) -> &mut Self {
		match self {
			Self::Local(g) => _ = g.create_new(create_new),
			Self::Sftp(g) => _ = g.create_new(create_new),
		};
		self
	}

	async fn new(scheme: SchemeRef<'_>) -> io::Result<Self> {
		Ok(match scheme {
			SchemeRef::Regular | SchemeRef::Search(_) => super::local::Gate::new(scheme).await?.into(),
			SchemeRef::Archive(_) => {
				Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem: archive"))?
			}
			SchemeRef::Sftp(_) => super::sftp::Gate::new(scheme).await?.into(),
		})
	}

	async fn open<P>(&self, path: P) -> io::Result<Self::File>
	where
		P: AsRef<Path>,
	{
		Ok(match self {
			Gate::Local(g) => g.open(path).await?.into(),
			Gate::Sftp(g) => g.open(path).await?.into(),
		})
	}

	fn read(&mut self, read: bool) -> &mut Self {
		match self {
			Self::Local(g) => _ = g.read(read),
			Self::Sftp(g) => _ = g.read(read),
		};
		self
	}

	fn truncate(&mut self, truncate: bool) -> &mut Self {
		match self {
			Self::Local(g) => _ = g.truncate(truncate),
			Self::Sftp(g) => _ = g.truncate(truncate),
		};
		self
	}

	fn write(&mut self, write: bool) -> &mut Self {
		match self {
			Self::Local(g) => _ = g.write(write),
			Self::Sftp(g) => _ = g.write(write),
		};
		self
	}
}
