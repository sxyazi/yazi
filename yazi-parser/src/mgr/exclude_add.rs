use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::CmdCow};

#[derive(Debug)]
pub struct ExcludeAddOpt {
	pub patterns: Vec<String>,
}

impl TryFrom<CmdCow> for ExcludeAddOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { patterns: c.take_seq::<SStr>().into_iter().map(|s| s.to_string()).collect() })
	}
}

impl FromLua for ExcludeAddOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ExcludeAddOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
