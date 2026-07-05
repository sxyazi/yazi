use mlua::{IntoLua, Lua, Table, Value};
use yazi_binding::{Composer, ComposerGet, ComposerSet};
use yazi_config::THEME;
use yazi_fs::file::FileRef;

pub(super) fn icon() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"match" => lua
				.create_function(|_, (_, file, opts): (Value, FileRef, Option<Table>)| {
					let hovered = opts.map(|t| t.raw_get("hovered")).transpose()?.unwrap_or(false);
					file.borrow(|f| Ok(THEME.icon.matches(f, hovered)))
				})?
				.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}
