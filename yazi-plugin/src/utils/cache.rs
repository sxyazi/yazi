use md5::{Digest, Md5};
use mlua::{Lua, Table};
use yazi_config::PREVIEW;

use super::Utils;
use crate::bindings::{Cast, Url};

impl Utils {
	pub(super) fn cache(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"cache_file",
			lua.create_function(|lua, data: mlua::String| {
				Url::cast(
					lua,
					PREVIEW.cache_dir.join(format!("{:x}", Md5::new_with_prefix(data).finalize())),
				)
			})?,
		)?;

		Ok(())
	}
}
