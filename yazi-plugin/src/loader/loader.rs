use std::{collections::HashMap, ops::Deref};

use anyhow::{Context, Result};
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
	pub async fn ensure(&self, name: &str) -> Result<()> {
		if self.cache.read().contains_key(name) {
			return Ok(());
		}

		// TODO: remove this
		let p = BOOT.plugin_dir.join(format!("{name}.yazi/main.lua"));
		let chunk = match fs::read(&p).await {
			Ok(b) => b,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
				static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
				if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
					yazi_proxy::AppProxy::notify(yazi_proxy::options::NotifyOpt {
						title:   "Deprecated entry file".to_owned(),
						content: format!(
							"The plugin entry file `init.lua` has been deprecated in v0.4.3 in favor of the new `main.lua`, and it will be fully removed in the next major version 0.5.

Please run `ya pack -m` to automatically migrate all plugins, or manually rename your `{name}.yazi/init.lua` to `{name}.yazi/main.lua`."
						),
						level:   yazi_proxy::options::NotifyLevel::Warn,
						timeout: std::time::Duration::from_secs(25),
					});
				}

				let p = BOOT.plugin_dir.join(format!("{name}.yazi/init.lua"));
				fs::read(&p).await.with_context(|| format!("Failed to load plugin from {p:?}"))?
			}
			Err(e) => Err(e).with_context(|| format!("Failed to load plugin from {p:?}"))?,
		};

		self.cache.write().insert(name.to_owned(), chunk.into());
		Ok(())
	}

	pub fn load(&self, lua: &Lua, id: &str) -> mlua::Result<Table> {
		let loaded: Table = lua.globals().raw_get::<Table>("package")?.raw_get("loaded")?;
		if let Ok(t) = loaded.raw_get::<Table>(id) {
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
		let loaded: Table = lua.globals().raw_get::<Table>("package")?.raw_get("loaded")?;
		loaded.raw_get(id)
	}

	pub fn load_with(&self, lua: &Lua, id: &str, chunk: &Chunk) -> mlua::Result<Table> {
		let loaded: Table = lua.globals().raw_get::<Table>("package")?.raw_get("loaded")?;
		if let Ok(t) = loaded.raw_get::<Table>(id) {
			return Ok(t);
		}

		let t: Table = lua.load(chunk.as_bytes()).set_name(id).call(())?;
		t.raw_set("_id", lua.create_string(id)?)?;

		loaded.raw_set(id, t.clone())?;
		Ok(t)
	}
}
