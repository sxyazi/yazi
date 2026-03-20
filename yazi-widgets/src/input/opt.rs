use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_shared::event::ActionCow;

use crate::input::InputEvent;

#[derive(Debug)]
pub struct InputOpt {
	pub cfg: InputCfg,
	pub tx:  mpsc::UnboundedSender<InputEvent>,
}

impl TryFrom<ActionCow> for InputOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Some(cfg) = a.take_any("cfg") else {
			bail!("Invalid 'cfg' in InputOpt");
		};

		let Some(tx) = a.take_any("tx") else {
			bail!("Invalid 'tx' in InputOpt");
		};

		Ok(Self { cfg, tx })
	}
}

impl FromLua for InputOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for InputOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
