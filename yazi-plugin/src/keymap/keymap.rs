use mlua::{ExternalResult, IntoLua, Lua, Value};
use yazi_binding::{Composer, ComposerGet};
use yazi_config::KEYMAP;

pub(crate) fn compose() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let layer = str::from_utf8(key)?.parse().into_lua_err()?;
		KEYMAP.section(layer).into_lua(lua)
	}

	Composer::new(get)
}
