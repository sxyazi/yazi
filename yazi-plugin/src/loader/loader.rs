use std::{borrow::Cow, collections::HashMap, ops::Deref};

use anyhow::Result;
use mlua::{ExternalError, Lua, Table};
use parking_lot::RwLock;
use tokio::fs;
use yazi_boot::BOOT;
use yazi_shared::RoCell;

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
			"archive" => &include_bytes!("../../preset/plugins/archive.lua")[..],
			"code" => include_bytes!("../../preset/plugins/code.lua"),
			"dds" => include_bytes!("../../preset/plugins/dds.lua"),
			"extract" => include_bytes!("../../preset/plugins/extract.lua"),
			"file" => include_bytes!("../../preset/plugins/file.lua"),
			"folder" => include_bytes!("../../preset/plugins/folder.lua"),
			"font" => include_bytes!("../../preset/plugins/font.lua"),
			"fzf" => include_bytes!("../../preset/plugins/fzf.lua"),
			"image" => include_bytes!("../../preset/plugins/image.lua"),
			"json" => include_bytes!("../../preset/plugins/json.lua"),
			"magick" => include_bytes!("../../preset/plugins/magick.lua"),
			"mime" => include_bytes!("../../preset/plugins/mime.lua"),
			"noop" => include_bytes!("../../preset/plugins/noop.lua"),
			"pdf" => include_bytes!("../../preset/plugins/pdf.lua"),
			"session" => include_bytes!("../../preset/plugins/session.lua"),
			"video" => include_bytes!("../../preset/plugins/video.lua"),
			"zoxide" => include_bytes!("../../preset/plugins/zoxide.lua"),
			_ => b"",
		};

		let b = if preset.is_empty() {
			Cow::Owned(fs::read(BOOT.plugin_dir.join(format!("{name}.yazi/init.lua"))).await?)
		} else {
			Cow::Borrowed(preset)
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
