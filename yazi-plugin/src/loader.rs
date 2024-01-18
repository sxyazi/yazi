use std::{borrow::Cow, collections::BTreeMap, ops::Deref};

use anyhow::{bail, Result};
use parking_lot::RwLock;
use tokio::fs;
use yazi_config::BOOT;
use yazi_shared::RoCell;

pub static LOADED: RoCell<Loader> = RoCell::new();

#[derive(Default)]
pub struct Loader {
	loaded: RwLock<BTreeMap<String, Vec<u8>>>,
}

impl Loader {
	#[inline]
	pub(super) fn init() { LOADED.with(Default::default); }

	pub async fn ensure(&self, name: &str) -> Result<()> {
		if self.loaded.read().contains_key(name) {
			return Ok(());
		}

		let path = BOOT.plugin_dir.join(format!("{name}.yazi/init.lua"));
		let b = fs::read(path).await.map(|v| v.into()).or_else(|_| {
			Ok(Cow::from(match name {
				"archive" => include_bytes!("../preset/plugins/archive.lua") as &[u8],
				"code" => include_bytes!("../preset/plugins/code.lua"),
				"file" => include_bytes!("../preset/plugins/file.lua"),
				"folder" => include_bytes!("../preset/plugins/folder.lua"),
				"image" => include_bytes!("../preset/plugins/image.lua"),
				"json" => include_bytes!("../preset/plugins/json.lua"),
				"mime" => include_bytes!("../preset/plugins/mime.lua"),
				"noop" => include_bytes!("../preset/plugins/noop.lua"),
				"pdf" => include_bytes!("../preset/plugins/pdf.lua"),
				"video" => include_bytes!("../preset/plugins/video.lua"),
				_ => bail!("plugin not found: {name}"),
			}))
		})?;

		self.loaded.write().insert(name.to_owned(), b.into_owned());
		Ok(())
	}
}

impl Deref for Loader {
	type Target = RwLock<BTreeMap<String, Vec<u8>>>;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.loaded }
}
