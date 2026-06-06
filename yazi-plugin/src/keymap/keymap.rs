use mlua::{IntoLua, Lua, Value};
use yazi_binding::{Composer, ComposerGet, ComposerSet, keymap::KeymapSection};
use yazi_shared::Layer;

pub fn compose() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"mgr" => KeymapSection::try_from(Layer::Mgr)?.into_lua(lua),
			b"tasks" => KeymapSection::try_from(Layer::Tasks)?.into_lua(lua),
			b"spot" => KeymapSection::try_from(Layer::Spot)?.into_lua(lua),
			b"pick" => KeymapSection::try_from(Layer::Pick)?.into_lua(lua),
			b"input" => KeymapSection::try_from(Layer::Input)?.into_lua(lua),
			b"confirm" => KeymapSection::try_from(Layer::Confirm)?.into_lua(lua),
			b"cmp" => KeymapSection::try_from(Layer::Cmp)?.into_lua(lua),
			b"help" => KeymapSection::try_from(Layer::Help)?.into_lua(lua),
			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}
