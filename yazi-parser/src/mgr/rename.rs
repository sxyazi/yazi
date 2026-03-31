use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::ActionCow};

#[derive(Debug)]
pub struct RenameForm {
	pub hovered: bool,
	pub force:   bool,
	pub empty:   SStr,
	pub cursor:  SStr,
}

impl From<ActionCow> for RenameForm {
	fn from(mut a: ActionCow) -> Self {
		Self {
			hovered: a.bool("hovered"),
			force:   a.bool("force"),
			empty:   a.take("empty").unwrap_or_default(),
			cursor:  a.take("cursor").unwrap_or_default(),
		}
	}
}

impl FromLua for RenameForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RenameForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
