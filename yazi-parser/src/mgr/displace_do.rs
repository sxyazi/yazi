use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::CmdCow, url::UrlBuf};

#[derive(Debug)]
pub struct DisplaceDoOpt {
	pub to:   std::io::Result<UrlBuf>,
	pub from: UrlBuf,
}

impl TryFrom<CmdCow> for DisplaceDoOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			opt
		} else {
			bail!("'opt' is required for DisplaceDoOpt");
		}
	}
}

impl FromLua for DisplaceDoOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for DisplaceDoOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
