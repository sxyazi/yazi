use std::io;

use yazi_fs::engine::{Attrs, Engine, FileBuilder};
use yazi_runner::provider::ProviderJob;
use yazi_shared::url::AsUrl;

use crate::engine::lua::{File, Lua};

#[derive(Clone, Copy, Default)]
pub struct Demand(yazi_fs::engine::Demand);

impl FileBuilder for Demand {
	type File = File;

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
		let engine = Lua::new(url.as_url()).await?;
		let job =
			ProviderJob::Open { url: engine.url.to_owned(), attrs: self.0.attrs, demand: self.0 };

		let pos = engine.call(job).await?.0?;
		Ok(File::new(engine.url, engine.run, pos, self.0))
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
