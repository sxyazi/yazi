use std::hash::Hash;

use mlua::{Function, Lua, Table};
use yazi_config::YAZI;
use yazi_fs::file::FileRef;
use yazi_shared::url::{UrlBuf, UrlLike};
use yazi_shim::Twox128;

use super::Utils;

impl Utils {
	pub(super) fn file_cache(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, t: Table| {
			let file: FileRef = t.raw_get("file")?;
			if file.url.parent() == Some(yazi_shared::url::Url::regular(&YAZI.preview.cache_dir)) {
				return Ok(None);
			}

			let hex = {
				let mut h = Twox128::default();
				file.hash(&mut h);
				t.raw_get("skip").unwrap_or(0usize).hash(&mut h);
				format!("{:x}", h.finish_128())
			};

			Ok(Some(UrlBuf::from(YAZI.preview.cache_dir.join(hex))))
		})
	}
}
