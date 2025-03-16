use std::{collections::HashMap, ops::Deref};

use anyhow::{Context, Result, bail};
use mlua::{ExternalError, Lua, Table};
use parking_lot::RwLock;
use tokio::fs;
use yazi_boot::BOOT;
use yazi_macro::plugin_preset as preset;
use yazi_shared::RoCell;

use super::Chunk;

pub static LOADER: RoCell<Loader> = RoCell::new();

pub struct Loader {
	cache: RwLock<HashMap<String, Chunk>>,
}

impl Deref for Loader {
	type Target = RwLock<HashMap<String, Chunk>>;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.cache }
}

impl Default for Loader {
	fn default() -> Self {
		let cache = HashMap::from_iter([
			("archive".to_owned(), preset!("plugins/archive").into()),
			("code".to_owned(), preset!("plugins/code").into()),
			("dds".to_owned(), preset!("plugins/dds").into()),
			("empty".to_owned(), preset!("plugins/empty").into()),
			("extract".to_owned(), preset!("plugins/extract").into()),
			("file".to_owned(), preset!("plugins/file").into()),
			("folder".to_owned(), preset!("plugins/folder").into()),
			("font".to_owned(), preset!("plugins/font").into()),
			("fzf".to_owned(), preset!("plugins/fzf").into()),
			("image".to_owned(), preset!("plugins/image").into()),
			("json".to_owned(), preset!("plugins/json").into()),
			("magick".to_owned(), preset!("plugins/magick").into()),
			("mime".to_owned(), preset!("plugins/mime").into()),
			("noop".to_owned(), preset!("plugins/noop").into()),
			("pdf".to_owned(), preset!("plugins/pdf").into()),
			("session".to_owned(), preset!("plugins/session").into()),
			("video".to_owned(), preset!("plugins/video").into()),
			("zoxide".to_owned(), preset!("plugins/zoxide").into()),
		]);
		Self { cache: RwLock::new(cache) }
	}
}

impl Loader {
	pub async fn ensure<F, T>(&self, name: &str, f: F) -> Result<T>
	where
		F: FnOnce(&Chunk) -> T,
	{
		if let Some(c) = self.cache.read().get(name) {
			return Self::compatible_or_error(name, c).map(|_| f(c));
		}

		let p = BOOT.plugin_dir.join(format!("{name}.yazi/main.lua"));
		let chunk =
			fs::read(&p).await.with_context(|| format!("Failed to load plugin from {p:?}"))?.into();

		let result = Self::compatible_or_error(name, &chunk);
		let inspect = f(&chunk);

		self.cache.write().insert(name.to_owned(), chunk);
		result.map(|_| inspect)
	}

	pub fn load(&self, lua: &Lua, id: &str) -> mlua::Result<Table> {
		let loaded: Table = lua.globals().raw_get::<Table>("package")?.raw_get("loaded")?;
		if let Ok(t) = loaded.raw_get(id) {
			return Ok(t);
		}

		let t: Table = match self.read().get(id) {
			Some(c) => lua.load(c.as_bytes()).set_name(id).call(())?,
			None => Err(format!("plugin `{id}` not found").into_lua_err())?,
		};

		t.raw_set("_id", lua.create_string(id)?)?;
		loaded.raw_set(id, t.clone())?;
		Ok(t)
	}

	pub fn try_load(&self, lua: &Lua, id: &str) -> mlua::Result<Table> {
		lua.globals().raw_get::<Table>("package")?.raw_get::<Table>("loaded")?.raw_get(id)
	}

	pub fn load_with(&self, lua: &Lua, id: &str, chunk: &Chunk) -> mlua::Result<Table> {
		let loaded: Table = lua.globals().raw_get::<Table>("package")?.raw_get("loaded")?;
		if let Ok(t) = loaded.raw_get(id) {
			return Ok(t);
		}

		let t: Table = lua.load(chunk.as_bytes()).set_name(id).call(())?;
		t.raw_set("_id", lua.create_string(id)?)?;

		loaded.raw_set(id, t.clone())?;
		Ok(t)
	}

	pub fn compatible_or_error(name: &str, chunk: &Chunk) -> Result<()> {
		if chunk.compatible() {
			return Ok(());
		}

		bail!(
			"Plugin `{name}` requires at least Yazi {}, but your current version is Yazi {}.",
			chunk.since,
			yazi_boot::actions::Actions::version()
		);
	}
}
