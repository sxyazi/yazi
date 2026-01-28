use mlua::{Function, Lua};
use twox_hash::XxHash3_128;
use yazi_widgets::CLIPBOARD;

use super::Utils;

impl Utils {
	pub(super) fn hash(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(move |_, s: mlua::String| async move {
			Ok(format!("{:x}", XxHash3_128::oneshot(&s.as_bytes())))
		})
	}

	pub(super) fn quote(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (s, unix): (mlua::String, Option<bool>)| {
			let b = s.as_bytes();
			let s = match unix {
				Some(true) => yazi_shared::shell::unix::escape_os_bytes(&b),
				Some(false) => yazi_shared::shell::windows::escape_os_bytes(&b),
				None => yazi_shared::shell::escape_os_bytes(&b),
			};
			lua.create_external_string(s)
		})
	}

	pub(super) fn clipboard(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, text: Option<String>| async move {
			if let Some(text) = text {
				CLIPBOARD.set(text).await;
				Ok(None)
			} else {
				Some(lua.create_external_string(CLIPBOARD.get().await)).transpose()
			}
		})
	}
}
