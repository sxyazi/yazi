use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::CmdCow};

#[derive(Debug)]
pub struct RenameOpt {
	pub hovered: bool,
	pub force:   bool,
	pub empty:   SStr,
	pub cursor:  SStr,
}

impl From<CmdCow> for RenameOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			hovered: c.bool("hovered"),
			force:   c.bool("force"),
			empty:   c.take("empty").unwrap_or_default(),
			cursor:  c.take("cursor").unwrap_or_default(),
		}
	}
}

impl FromLua for RenameOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RenameOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
