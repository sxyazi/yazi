use md5::{Digest, Md5};
use mlua::{Lua, Table};
use yazi_config::PREVIEW;

use super::Utils;
use crate::{bindings::Cast, file::FileRef, url::Url};

impl Utils {
	pub(super) fn cache(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"file_cache",
			lua.create_function(|lua, t: Table| {
				let file: FileRef = t.raw_get("file")?;
				if file.url().parent() == Some(&PREVIEW.cache_dir) {
					return Ok(None);
				}

				let hex = {
					let mut digest = Md5::new_with_prefix(file.url().as_os_str().as_encoded_bytes());
					digest.update(format!("//{:?}//{}", file.cha.mtime, t.raw_get("skip").unwrap_or(0)));
					format!("{:x}", digest.finalize())
				};

				Some(Url::cast(lua, PREVIEW.cache_dir.join(hex))).transpose()
			})?,
		)?;

		Ok(())
	}
}
