use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::mgr::FilterOpt;
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug, Default)]
pub struct FilterForm {
	pub opt: FilterOpt,
}

impl TryFrom<ActionCow> for FilterForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: if let Some(opt) = a.take_any("opt") { opt } else { a.try_into()? } })
	}
}

impl FromLua for FilterForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for FilterForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
