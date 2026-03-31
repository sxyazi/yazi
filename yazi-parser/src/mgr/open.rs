use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::mgr::OpenOpt;
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug)]
pub struct OpenForm {
	pub opt: OpenOpt,
}

impl From<OpenOpt> for OpenForm {
	fn from(opt: OpenOpt) -> Self { Self { opt } }
}

impl TryFrom<ActionCow> for OpenForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: if let Some(opt) = a.take_any("opt") { opt } else { a.try_into()? } })
	}
}

impl FromLua for OpenForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for OpenForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
