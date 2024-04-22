use std::{borrow::Cow, collections::HashMap, ops::Deref};

use anyhow::{bail, Result};
use mlua::{ExternalError, Table};
use parking_lot::RwLock;
use tokio::fs;
use yazi_boot::BOOT;
use yazi_shared::RoCell;

use crate::LUA;

pub static LOADER: RoCell<Loader> = RoCell::new();

#[derive(Default)]
pub struct Loader {
	cache: RwLock<HashMap<String, Cow<'static, [u8]>>>,
}

impl Loader {
	pub async fn ensure(&self, name: &str) -> Result<()> {
		if self.cache.read().contains_key(name) {
			return Ok(());
		}

		let b = match name {
			"dds" => Some(&include_bytes!("../../preset/plugins/dds.lua")[..]),
			"noop" => Some(&include_bytes!("../../preset/plugins/noop.lua")[..]),
			"session" => Some(&include_bytes!("../../preset/plugins/session.lua")[..]),
			_ => None,
		};
		if let Some(b) = b {
			self.cache.write().insert(name.to_owned(), Cow::Borrowed(b));
			return Ok(());
		}

		let path = BOOT.plugin_dir.join(format!("{name}.yazi/init.lua"));
		let b = fs::read(path).await.map(|v| v.into()).or_else(|_| {
			Ok(Cow::from(match name {
				"archive" => &include_bytes!("../../preset/plugins/archive.lua")[..],
				"code" => include_bytes!("../../preset/plugins/code.lua"),
				"file" => include_bytes!("../../preset/plugins/file.lua"),
				"folder" => include_bytes!("../../preset/plugins/folder.lua"),
				"fzf" => include_bytes!("../../preset/plugins/fzf.lua"),
				"image" => include_bytes!("../../preset/plugins/image.lua"),
				"json" => include_bytes!("../../preset/plugins/json.lua"),
				"mime" => include_bytes!("../../preset/plugins/mime.lua"),
				"pdf" => include_bytes!("../../preset/plugins/pdf.lua"),
				"video" => include_bytes!("../../preset/plugins/video.lua"),
				"zoxide" => include_bytes!("../../preset/plugins/zoxide.lua"),
				_ => bail!("plugin not found: {name}"),
			}))
		})?;

		self.cache.write().insert(name.to_owned(), b);
		Ok(())
	}

	pub fn load(&self, name: &str) -> mlua::Result<Table> {
		let globals = LUA.globals();
		let loaded: Table = globals.raw_get::<_, Table>("package")?.raw_get("loaded")?;
		if let Ok(t) = loaded.raw_get::<_, Table>(name) {
			return Ok(t);
		}

		let t: Table = match self.read().get(name) {
			Some(b) => LUA.load(b.as_ref()).call(())?,
			None => Err(format!("plugin `{name}` not found").into_lua_err())?,
		};

		t.raw_set("_name", LUA.create_string(name)?)?;
		loaded.raw_set(name, t.clone())?;
		Ok(t)
	}
}

impl Deref for Loader {
	type Target = RwLock<HashMap<String, Cow<'static, [u8]>>>;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.cache }
}
