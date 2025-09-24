use std::{io, path::Path};

use yazi_shared::scheme::SchemeRef;

use crate::{cha::Cha, provider::FileBuilder};

#[derive(Default)]
pub struct Gate(tokio::fs::OpenOptions);

impl FileBuilder for Gate {
	type File = tokio::fs::File;

	fn append(&mut self, append: bool) -> &mut Self {
		self.0.append(append);
		self
	}

	fn cha(&mut self, cha: Cha) -> &mut Self {
		#[cfg(unix)]
		self.0.mode(cha.mode.bits() as _);
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

	async fn new(scheme: SchemeRef<'_>) -> io::Result<Self> {
		if scheme.is_virtual() {
			Err(io::Error::new(io::ErrorKind::InvalidInput, "Not a local filesystem"))?
		} else {
			Ok(Self::default())
		}
	}

	async fn open<P>(&self, path: P) -> io::Result<Self::File>
	where
		P: AsRef<Path>,
	{
		self.0.open(path).await
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
