use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::mgr::FindDoOpt;
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug)]
pub struct FindDoForm {
	pub opt: FindDoOpt,
}

impl TryFrom<ActionCow> for FindDoForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: if let Some(opt) = a.take_any("opt") { opt } else { a.try_into()? } })
	}
}

impl FromLua for FindDoForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for FindDoForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
