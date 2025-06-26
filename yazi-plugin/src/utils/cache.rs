use std::hash::Hash;

use mlua::{Function, Lua, Table};
use yazi_binding::Url;
use yazi_config::YAZI;

use super::Utils;
use crate::{Twox128, file::FileRef};

impl Utils {
	pub(super) fn file_cache(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, t: Table| {
			let file: FileRef = t.raw_get("file")?;
			if file.url.parent() == Some(&YAZI.preview.cache_dir) {
				return Ok(None);
			}

			let hex = {
				let mut h = Twox128::default();
				file.hash(&mut h);
				t.raw_get("skip").unwrap_or(0usize).hash(&mut h);
				format!("{:x}", h.finish_128())
			};

			Ok(Some(Url::new(YAZI.preview.cache_dir.join(hex))))
		})
	}
}
