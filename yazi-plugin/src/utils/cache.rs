use mlua::{Function, Lua, Table};
use twox_hash::XxHash3_128;
use yazi_binding::Url;
use yazi_config::YAZI;

use super::Utils;
use crate::file::FileRef;

impl Utils {
	pub(super) fn file_cache(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, t: Table| {
			let file: FileRef = t.raw_get("file")?;
			if file.url.parent() == Some(&YAZI.preview.cache_dir) {
				return Ok(None);
			}

			let hex = {
				let mut h = XxHash3_128::new();
				h.write(file.url.as_os_str().as_encoded_bytes());
				h.write(format!("//{:?}//{}", file.cha.mtime, t.raw_get("skip").unwrap_or(0)).as_bytes());
				format!("{:x}", h.finish_128())
			};

			Ok(Some(Url::new(YAZI.preview.cache_dir.join(hex))))
		})
	}
}
