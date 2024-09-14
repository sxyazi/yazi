use std::{borrow::Cow, collections::HashMap, ops::Deref};

use anyhow::{Context, Result};
use mlua::{ExternalError, Lua, Table};
use parking_lot::RwLock;
use tokio::fs;
use yazi_boot::BOOT;
use yazi_shared::RoCell;

use crate::preset;

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

		let preset = match name {
			"archive" => preset!("plugins/archive"),
			"code" => preset!("plugins/code"),
			"dds" => preset!("plugins/dds"),
			"empty" => preset!("plugins/empty"),
			"extract" => preset!("plugins/extract"),
			"file" => preset!("plugins/file"),
			"folder" => preset!("plugins/folder"),
			"font" => preset!("plugins/font"),
			"fzf" => preset!("plugins/fzf"),
			"image" => preset!("plugins/image"),
			"json" => preset!("plugins/json"),
			"magick" => preset!("plugins/magick"),
			"mime" => preset!("plugins/mime"),
			"noop" => preset!("plugins/noop"),
			"pdf" => preset!("plugins/pdf"),
			"session" => preset!("plugins/session"),
			"video" => preset!("plugins/video"),
			"zoxide" => preset!("plugins/zoxide"),
			_ => Default::default(),
		};

		let b = if preset.is_empty() {
			let p = BOOT.plugin_dir.join(format!("{name}.yazi/init.lua"));
			Cow::Owned(fs::read(&p).await.with_context(|| format!("failed to load plugin from {p:?}"))?)
		} else {
			preset.into()
		};

		self.cache.write().insert(name.to_owned(), b);
		Ok(())
	}

	pub fn load<'a>(&self, lua: &'a Lua, id: &str) -> mlua::Result<Table<'a>> {
		let loaded: Table = lua.globals().raw_get::<_, Table>("package")?.raw_get("loaded")?;
		if let Ok(t) = loaded.raw_get::<_, Table>(id) {
			return Ok(t);
		}

		let t: Table = match self.read().get(id) {
			Some(b) => lua.load(b.as_ref()).set_name(id).call(())?,
			None => Err(format!("plugin `{id}` not found").into_lua_err())?,
		};

		t.raw_set("_id", lua.create_string(id)?)?;
		loaded.raw_set(id, t.clone())?;
		Ok(t)
	}
}

impl Deref for Loader {
	type Target = RwLock<HashMap<String, Cow<'static, [u8]>>>;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.cache }
}
