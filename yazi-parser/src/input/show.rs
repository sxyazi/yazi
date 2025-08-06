use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_shared::{errors::InputError, event::CmdCow};

#[derive(Debug)]
pub struct ShowOpt {
	pub cfg: InputCfg,
	pub tx:  mpsc::UnboundedSender<Result<String, InputError>>,
}

impl TryFrom<CmdCow> for ShowOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(cfg) = c.take_any("cfg") else {
			bail!("Invalid 'cfg' argument in ShowOpt");
		};

		let Some(tx) = c.take_any("tx") else {
			bail!("Invalid 'tx' argument in ShowOpt");
		};

		Ok(Self { cfg, tx })
	}
}

impl FromLua for ShowOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ShowOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
