use std::io;

use yazi_shared::url::AsUrl;

use crate::provider::{Attrs, FileBuilder};

#[derive(Default)]
pub struct Gate(tokio::fs::OpenOptions);

impl FileBuilder for Gate {
	type File = tokio::fs::File;

	fn append(&mut self, append: bool) -> &mut Self {
		self.0.append(append);
		self
	}

	fn attrs(&mut self, _attrs: Attrs) -> &mut Self {
		#[cfg(unix)]
		if let Some(mode) = _attrs.mode {
			self.0.mode(mode.bits() as _);
		}
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
		if let Some(path) = url.as_local() {
			self.0.open(path).await
		} else {
			Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Not a local URL: {url:?}")))
		}
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
