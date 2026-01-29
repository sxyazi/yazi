use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct CasefyOpt {
	pub upper: bool,
}

impl From<CmdCow> for CasefyOpt {
	fn from(c: CmdCow) -> Self { Self { upper: c.str(0) == "upper" } }
}

impl FromLua for CasefyOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CasefyOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

impl CasefyOpt {
	pub fn transform(&self, s: &str) -> String {
		if self.upper { s.to_ascii_uppercase() } else { s.to_ascii_lowercase() }
	}
}
