use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_config::popup::ConfirmCfg;
use yazi_shared::{CompletionToken, event::CmdCow};

#[derive(Debug)]
pub struct ShowOpt {
	pub cfg:   ConfirmCfg,
	pub token: CompletionToken,
}

impl TryFrom<CmdCow> for ShowOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(cfg) = c.take_any("cfg") else {
			bail!("Invalid 'cfg' in ShowOpt");
		};

		let Some(token) = c.take_any("token") else {
			bail!("Invalid 'token' in ShowOpt");
		};

		Ok(Self { cfg, token })
	}
}

impl From<Box<Self>> for ShowOpt {
	fn from(value: Box<Self>) -> Self { *value }
}

impl FromLua for ShowOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ShowOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
