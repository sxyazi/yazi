use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use tokio::sync::mpsc;
use yazi_config::popup::PickCfg;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug)]
pub struct ShowForm {
	pub cfg: PickCfg,
	pub tx:  mpsc::UnboundedSender<Option<usize>>,
}

impl<C> FromAction<C> for ShowForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		let Some(cfg) = a.take_any("cfg") else {
			bail!("Invalid 'cfg' in ShowForm");
		};

		let Some(tx) = a.take_any("tx") else {
			bail!("Invalid 'tx' in ShowForm");
		};

		Ok(Self { cfg, tx })
	}
}

impl FromLua for ShowForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ShowForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
