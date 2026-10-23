use mlua::{IntoLua, Lua, Value};
use yazi_binding::{Composer, ComposerGet};
use yazi_config::YAZI;

pub(super) fn plugin() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"fetchers" => YAZI.plugin.fetchers.into_lua(lua),
			b"spotters" => YAZI.plugin.spotters.into_lua(lua),
			b"preloaders" => YAZI.plugin.preloaders.into_lua(lua),
			b"previewers" => YAZI.plugin.previewers.into_lua(lua),
			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}
