use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use tokio::sync::mpsc;
use yazi_config::popup::PickCfg;
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct ShowOpt {
	pub cfg: PickCfg,
	pub tx:  mpsc::UnboundedSender<Option<usize>>,
}

impl TryFrom<ActionCow> for ShowOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Some(cfg) = a.take_any("cfg") else {
			bail!("Invalid 'cfg' in ShowOpt");
		};

		let Some(tx) = a.take_any("tx") else {
			bail!("Invalid 'tx' in ShowOpt");
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
