use std::io;

use yazi_fs::provider::{Attrs, FileBuilder};
use yazi_shared::{scheme::SchemeKind, url::AsUrl};

#[derive(Clone, Copy, Default)]
pub struct Gate {
	pub(super) append:     bool,
	pub(super) attrs:      Attrs,
	pub(super) create:     bool,
	pub(super) create_new: bool,
	pub(super) read:       bool,
	pub(super) truncate:   bool,
	pub(super) write:      bool,
}

impl FileBuilder for Gate {
	type File = super::RwFile;

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

	async fn open<U>(&self, url: U) -> io::Result<Self::File>
	where
		U: AsUrl,
	{
		let url = url.as_url();
		Ok(match url.kind() {
			SchemeKind::Regular | SchemeKind::Search => {
				self.build::<yazi_fs::provider::local::Gate>().open(url).await?.into()
			}
			SchemeKind::Archive => {
				Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem: archive"))?
			}
			SchemeKind::Sftp => self.build::<super::sftp::Gate>().open(url).await?.into(),
		})
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

impl Gate {
	fn build<T: FileBuilder>(self) -> T {
		let mut gate = T::default();
		if self.append {
			gate.append(true);
		}
		gate.attrs(self.attrs);
		if self.create {
			gate.create(true);
		}
		if self.create_new {
			gate.create_new(true);
		}
		if self.read {
			gate.read(true);
		}
		if self.truncate {
			gate.truncate(true);
		}
		if self.write {
			gate.write(true);
		}
		gate
	}
}
