use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_config::popup::ConfirmCfg;
use yazi_shared::{CompletionToken, event::{ActionCow, FromAction}};

#[derive(Debug)]
pub struct ShowForm {
	pub cfg:   ConfirmCfg,
	pub token: CompletionToken,
}

impl<C> FromAction<C> for ShowForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		let Some(cfg) = a.take_any("cfg") else {
			bail!("Invalid 'cfg' in ShowForm");
		};

		let Some(token) = a.take_any("token") else {
			bail!("Invalid 'token' in ShowForm");
		};

		Ok(Self { cfg, token })
	}
}

impl From<Box<Self>> for ShowForm {
	fn from(value: Box<Self>) -> Self { *value }
}

impl FromLua for ShowForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ShowForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
