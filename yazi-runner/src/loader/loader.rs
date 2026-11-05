use std::{borrow::Cow, ops::Deref};

use anyhow::{Context, Result, bail, ensure};
use hashbrown::HashMap;
use mlua::{ExternalError, Lua, Table, chunk::ChunkMode};
use parking_lot::RwLock;
use yazi_fs::{Xdg, engine::local::Local};
use yazi_macro::plugin_preset as preset;
use yazi_shim::{CompactString, bytes::BytesExt, cell::RoCell, log::LOG_LEVEL};

use super::Chunk;

pub static LOADER: RoCell<Loader> = RoCell::new();

pub struct Loader {
	cache: RwLock<HashMap<CompactString, Chunk>>,
}

impl Deref for Loader {
	type Target = RwLock<HashMap<CompactString, Chunk>>;

	fn deref(&self) -> &Self::Target { &self.cache }
}

impl Default for Loader {
	fn default() -> Self {
		let cache = HashMap::from_iter([
			// Plugins
			("archive".into(), preset!("plugins/archive").into()),
			("clipboard".into(), preset!("plugins/clipboard").into()),
			("code".into(), preset!("plugins/code").into()),
			("dds".into(), preset!("plugins/dds").into()),
			("dnd".into(), preset!("plugins/dnd").into()),
			("empty".into(), preset!("plugins/empty").into()),
			("error".into(), preset!("plugins/error").into()),
			("extract".into(), preset!("plugins/extract").into()),
			("fd".into(), preset!("plugins/fd").into()),
			("file".into(), preset!("plugins/file").into()),
			("folder".into(), preset!("plugins/folder").into()),
			("font".into(), preset!("plugins/font").into()),
			("fzf".into(), preset!("plugins/fzf").into()),
			("go".into(), preset!("plugins/go").into()),
			("image".into(), preset!("plugins/image").into()),
			("init".into(), preset!("plugins/init").into()),
			("json".into(), preset!("plugins/json").into()),
			("magick".into(), preset!("plugins/magick").into()),
			("mime".into(), preset!("plugins/mime").into()),
			("mime.dir".into(), preset!("plugins/mime-dir").into()),
			("mime.local".into(), preset!("plugins/mime-local").into()),
			("mime.remote".into(), preset!("plugins/mime-remote").into()),
			("mime.trash".into(), preset!("plugins/mime-trash").into()),
			("multi".into(), preset!("plugins/multi").into()),
			("noop".into(), preset!("plugins/noop").into()),
			("pdf".into(), preset!("plugins/pdf").into()),
			("place".into(), preset!("plugins/place").into()),
			("rg".into(), preset!("plugins/rg").into()),
			("search".into(), preset!("plugins/search").into()),
			("session".into(), preset!("plugins/session").into()),
			("svg".into(), preset!("plugins/svg").into()),
			("trash".into(), preset!("plugins/trash").into()),
			("vfs".into(), preset!("plugins/vfs").into()),
			("video".into(), preset!("plugins/video").into()),
			("zoxide".into(), preset!("plugins/zoxide").into()),
			// Components
			("app".into(), [][..].into()),
			("backdrop".into(), [][..].into()),
			("current".into(), [][..].into()),
			("entity".into(), [][..].into()),
			("header".into(), [][..].into()),
			("linemode".into(), [][..].into()),
			("marker".into(), [][..].into()),
			("markers".into(), [][..].into()),
			("modal".into(), [][..].into()),
			("parent".into(), [][..].into()),
			("preview".into(), [][..].into()),
			("progress".into(), [][..].into()),
			("rail".into(), [][..].into()),
			("rails".into(), [][..].into()),
			("root".into(), [][..].into()),
			("status".into(), [][..].into()),
			("tab".into(), [][..].into()),
			("tabs".into(), [][..].into()),
			("tasks".into(), [][..].into()),
			("tip".into(), [][..].into()),
			// Reserved
			("history".into(), [][..].into()),
			("inline".into(), [][..].into()),
			("none".into(), [][..].into()),
			("null".into(), [][..].into()),
			("sftp".into(), [][..].into()),
			("undo".into(), [][..].into()),
		]);
		Self { cache: RwLock::new(cache) }
	}
}

impl Loader {
	pub async fn ensure<F, T>(&self, name: &str, f: F) -> Result<T>
	where
		F: FnOnce(&Chunk) -> T,
	{
		let (full, plugin, entry) = Self::explode_name_parts(name)?;
		if let Some(c) = self.cache.read().get(full) {
			return Self::compatible_or_error(full, c).map(|_| f(c));
		}

		let p = Xdg::config_dir().join(format!("plugins/{plugin}.yazi/{entry}.lua"));
		let chunk = Local::regular(&p)
			.read()
			.await
			.with_context(|| format!("Failed to load plugin from {p:?}"))?
			.into();

		let result = Self::compatible_or_error(full, &chunk);
		let inspect = f(&chunk);

		self.cache.write().insert(full.into(), chunk);
		result.map(|_| inspect)
	}

	pub(crate) async fn load(&self, lua: &Lua, name: &str) -> mlua::Result<Table> {
		let (name, ..) = Self::explode_name_parts(name)?;

		let loaded: Table = lua.globals().raw_get::<Table>("package")?.raw_get("loaded")?;
		if let Ok(t) = loaded.raw_get(name) {
			return Ok(t);
		}

		let t = self.load_new(lua, name).await?;
		t.raw_set("_id", lua.create_string(name)?)?;

		loaded.raw_set(name, t.clone())?;
		Ok(t)
	}

	async fn load_new(&self, lua: &Lua, name: &str) -> mlua::Result<Table> {
		let mut mode = ChunkMode::Text;
		let f = match self.cache.read().get(name) {
			Some(c) => {
				mode = c.mode;
				lua.load(c).set_name(name).into_function()
			}
			None => Err(format!("Plugin `{name}` not found").into_lua_err()),
		}?;

		if mode != ChunkMode::Binary {
			let b = f.dump(LOG_LEVEL.get().is_none());
			if let Some(c) = self.cache.write().get_mut(name) {
				c.mode = ChunkMode::Binary;
				c.bytes = Cow::Owned(b);
			}
		}

		f.call_async(()).await
	}

	pub fn load_chunk(&self, lua: &Lua, name: &str, chunk: &Chunk) -> mlua::Result<Table> {
		let (name, ..) = Self::explode_name_parts(name)?;

		let loaded: Table = lua.globals().raw_get::<Table>("package")?.raw_get("loaded")?;
		if let Ok(t) = loaded.raw_get(name) {
			return Ok(t);
		}

		let t: Table = lua.load(chunk).set_name(name).call(())?;
		t.raw_set("_id", lua.create_string(name)?)?;

		loaded.raw_set(name, t.clone())?;
		Ok(t)
	}

	pub fn try_load(&self, lua: &Lua, name: &str) -> mlua::Result<Table> {
		let (name, ..) = Self::explode_name_parts(name)?;
		lua.globals().raw_get::<Table>("package")?.raw_get::<Table>("loaded")?.raw_get(name)
	}

	pub fn compatible_or_error(name: &str, chunk: &Chunk) -> Result<()> {
		if chunk.compatible() {
			return Ok(());
		}

		bail!(
			"Plugin `{name}` requires at least Yazi {}, but your current version is Yazi {}.",
			chunk.since,
			yazi_version::version_long()
		);
	}

	fn explode_name_parts(name: &str) -> anyhow::Result<(&str, &str, &str)> {
		let name = name.strip_suffix(".main").unwrap_or(name);
		let (plugin, entry) =
			if let Some((a, b)) = name.split_once(".") { (a, b) } else { (name, "main") };

		ensure!(plugin.as_bytes().kebab_cased(), "Plugin name `{plugin}` must be in kebab-case");
		ensure!(entry.as_bytes().kebab_cased(), "Entry name `{entry}` must be in kebab-case");
		Ok((name, plugin, entry))
	}
}
