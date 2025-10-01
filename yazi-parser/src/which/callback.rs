use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use tokio::sync::mpsc;
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct CallbackOpt {
	pub tx:  mpsc::Sender<usize>,
	pub idx: usize,
}

impl TryFrom<CmdCow> for CallbackOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(tx) = c.take_any("tx") else {
			bail!("Invalid 'tx' argument in CallbackOpt");
		};

		let Ok(idx) = c.first() else {
			bail!("Invalid 'idx' argument in CallbackOpt");
		};

		Ok(Self { tx, idx })
	}
}

impl FromLua for CallbackOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CallbackOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
