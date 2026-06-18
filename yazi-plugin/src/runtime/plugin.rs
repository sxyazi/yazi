use mlua::{IntoLua, Lua, Value};
use yazi_binding::{Composer, ComposerGet, ComposerSet};
use yazi_config::YAZI;

pub(super) fn plugin() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"fetchers" => YAZI.plugin.fetchers.into_lua(lua),
			b"spotters" => YAZI.plugin.spotters.into_lua(lua),
			b"preloaders" => YAZI.plugin.preloaders.into_lua(lua),
			b"previewers" => YAZI.plugin.previewers.into_lua(lua),
			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}
