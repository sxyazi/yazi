use std::{borrow::Cow, collections::BTreeMap, ops::Deref};

use anyhow::{bail, Result};
use mlua::{ExternalError, Table};
use parking_lot::RwLock;
use tokio::fs;
use yazi_boot::BOOT;
use yazi_shared::RoCell;

use crate::LUA;

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

	pub fn load(&self, name: &str) -> mlua::Result<Table> {
		let globals = LUA.globals();
		let loaded: Table = globals.raw_get::<_, Table>("package")?.raw_get("loaded")?;
		if let Ok(t) = loaded.raw_get::<_, Table>(name) {
			return Ok(t);
		}

		globals.raw_set("YAZI_PLUGIN_NAME", LUA.create_string(name)?)?;
		globals.raw_set("YAZI_SYNC_CALLS", 0)?;
		let t: Table = match self.read().get(name) {
			Some(b) => LUA.load(b).call(())?,
			None => Err(format!("plugin `{name}` not found").into_lua_err())?,
		};

		loaded.raw_set(name, t.clone())?;
		Ok(t)
	}
}

impl Deref for Loader {
	type Target = RwLock<BTreeMap<String, Vec<u8>>>;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.loaded }
}
