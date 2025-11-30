use std::{borrow::Cow, ops::Deref};

use anyhow::{Context, Result, bail, ensure};
use hashbrown::HashMap;
use mlua::{ChunkMode, ExternalError, Lua, Table};
use parking_lot::RwLock;
use yazi_boot::BOOT;
use yazi_fs::provider::local::Local;
use yazi_macro::plugin_preset as preset;
use yazi_shared::{BytesExt, LOG_LEVEL, RoCell};

use super::Chunk;

pub static LOADER: RoCell<Loader> = RoCell::new();

pub struct Loader {
	cache: RwLock<HashMap<String, Chunk>>,
}

impl Deref for Loader {
	type Target = RwLock<HashMap<String, Chunk>>;

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
			("mime".to_owned(), preset!("plugins/mime").into()), // TODO: remove this
			("mime.dir".to_owned(), preset!("plugins/mime-dir").into()),
			("mime.local".to_owned(), preset!("plugins/mime-local").into()),
			("mime.remote".to_owned(), preset!("plugins/mime-remote").into()),
			("noop".to_owned(), preset!("plugins/noop").into()),
			("null".to_owned(), preset!("plugins/null").into()),
			("pdf".to_owned(), preset!("plugins/pdf").into()),
			("session".to_owned(), preset!("plugins/session").into()),
			("svg".to_owned(), preset!("plugins/svg").into()),
			("vfs".to_owned(), preset!("plugins/vfs").into()),
			("video".to_owned(), preset!("plugins/video").into()),
			("zoxide".to_owned(), preset!("plugins/zoxide").into()),
		]);
		Self { cache: RwLock::new(cache) }
	}
}

impl Loader {
	pub async fn ensure<F, T>(&self, id: &str, f: F) -> Result<T>
	where
		F: FnOnce(&Chunk) -> T,
	{
		let (id, plugin, entry) = Self::normalize_id(id)?;
		if let Some(c) = self.cache.read().get(id) {
			return Self::compatible_or_error(id, c).map(|_| f(c));
		}

		let p = BOOT.plugin_dir.join(format!("{plugin}.yazi/{entry}.lua"));
		let chunk = Local::regular(&p)
			.read()
			.await
			.with_context(|| format!("Failed to load plugin from {p:?}"))?
			.into();

		let result = Self::compatible_or_error(id, &chunk);
		let inspect = f(&chunk);

		self.cache.write().insert(id.to_owned(), chunk);
		result.map(|_| inspect)
	}

	pub fn load(&self, lua: &Lua, id: &str) -> mlua::Result<Table> {
		let (id, ..) = Self::normalize_id(id)?;

		let loaded: Table = lua.globals().raw_get::<Table>("package")?.raw_get("loaded")?;
		if let Ok(t) = loaded.raw_get(id) {
			return Ok(t);
		}

		let t = self.load_once(lua, id)?;
		t.raw_set("_id", lua.create_string(id)?)?;

		loaded.raw_set(id, t.clone())?;
		Ok(t)
	}

	pub fn load_once(&self, lua: &Lua, id: &str) -> mlua::Result<Table> {
		let (id, ..) = Self::normalize_id(id)?;

		let mut mode = ChunkMode::Text;
		let f = match self.cache.read().get(id) {
			Some(c) => {
				mode = c.mode;
				lua.load(c).set_name(id).into_function()
			}
			None => Err(format!("Plugin `{id}` not found").into_lua_err()),
		}?;

		if mode != ChunkMode::Binary {
			let b = f.dump(LOG_LEVEL.get().is_none());
			if let Some(c) = self.cache.write().get_mut(id) {
				c.mode = ChunkMode::Binary;
				c.bytes = Cow::Owned(b);
			}
		}

		f.call(())
	}

	pub fn try_load(&self, lua: &Lua, id: &str) -> mlua::Result<Table> {
		let (id, ..) = Self::normalize_id(id)?;
		lua.globals().raw_get::<Table>("package")?.raw_get::<Table>("loaded")?.raw_get(id)
	}

	pub fn load_with(&self, lua: &Lua, id: &str, chunk: &Chunk) -> mlua::Result<Table> {
		let (id, ..) = Self::normalize_id(id)?;

		let loaded: Table = lua.globals().raw_get::<Table>("package")?.raw_get("loaded")?;
		if let Ok(t) = loaded.raw_get(id) {
			return Ok(t);
		}

		let t: Table = lua.load(chunk).set_name(id).call(())?;
		t.raw_set("_id", lua.create_string(id)?)?;

		loaded.raw_set(id, t.clone())?;
		Ok(t)
	}

	pub fn compatible_or_error(id: &str, chunk: &Chunk) -> Result<()> {
		if chunk.compatible() {
			return Ok(());
		}

		bail!(
			"Plugin `{id}` requires at least Yazi {}, but your current version is Yazi {}.",
			chunk.since,
			yazi_boot::actions::Actions::version()
		);
	}

	pub fn normalize_id(id: &str) -> anyhow::Result<(&str, &str, &str)> {
		let id = id.trim_end_matches(".main");
		let (plugin, entry) = if let Some((a, b)) = id.split_once(".") { (a, b) } else { (id, "main") };

		ensure!(plugin.as_bytes().kebab_cased(), "Plugin name `{plugin}` must be in kebab-case");
		ensure!(entry.as_bytes().kebab_cased(), "Entry name `{entry}` must be in kebab-case");
		Ok((id, plugin, entry))
	}
}
