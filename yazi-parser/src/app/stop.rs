use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::{ActionCow, Replier};

#[derive(Clone, Debug, Default)]
pub struct StopForm {
	pub replier: Option<Replier>,
}

impl TryFrom<ActionCow> for StopForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { replier: a.take_replier() })
	}
}

impl FromLua for StopForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for StopForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
