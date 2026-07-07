use std::io;

use yazi_fs::provider::{Attrs, FileBuilder};
use yazi_shared::url::{AsUrl, Url};

use crate::config::{ServiceRclone, Vfs};

#[derive(Clone, Copy, Default)]
pub struct Gate(crate::provider::Gate);

impl FileBuilder for Gate {
	type File = super::File;

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
		let (path, (_, config)) = match url {
			Url::Rclone { loc, domain } => {
				(loc.as_inner(), Vfs::service::<&ServiceRclone>(domain).await?)
			}
			_ => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Not an rclone URL: {url:?}")))?,
		};

		if self.0.write || self.0.append || self.0.truncate || self.0.create || self.0.create_new {
			return Err(super::read_only());
		}

		let target = super::target(config, path)?;
		let item = super::stat(config, &target).await?;
		if item.is_dir {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "Is a directory"));
		}

		Ok(super::File::new(config, target, item.size))
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
