use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct HardlinkOpt {
	pub force:  bool,
	pub follow: bool,
}

impl From<ActionCow> for HardlinkOpt {
	fn from(a: ActionCow) -> Self { Self { force: a.bool("force"), follow: a.bool("follow") } }
}

impl FromLua for HardlinkOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for HardlinkOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
