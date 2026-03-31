use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use tokio::sync::mpsc;
use yazi_shared::CompletionToken;

#[derive(Debug)]
pub struct StopForm {
	pub tx:    mpsc::UnboundedSender<(bool, CompletionToken)>,
	pub token: CompletionToken,
}

impl FromLua for StopForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for StopForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
