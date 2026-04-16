use mlua::{IntoLua, Lua, Value};
use yazi_binding::{Composer, ComposerGet, ComposerSet, config::{Fetchers, Preloaders, Previewers, Spotters}};

pub(super) fn plugin() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"fetchers" => Fetchers.into_lua(lua),
			b"spotters" => Spotters.into_lua(lua),
			b"preloaders" => Preloaders.into_lua(lua),
			b"previewers" => Previewers.into_lua(lua),
			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}
