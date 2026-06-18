use mlua::{ExternalResult, IntoLua, Lua, Value};
use yazi_binding::{Composer, ComposerGet, ComposerSet};
use yazi_config::KEYMAP;

pub fn compose() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let layer = str::from_utf8(key)?.parse().into_lua_err()?;
		KEYMAP.section(layer).into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}
