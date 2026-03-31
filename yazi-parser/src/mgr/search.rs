use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::mgr::SearchOpt;
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug)]
pub struct SearchForm {
	pub opt: SearchOpt,
}

impl TryFrom<ActionCow> for SearchForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: if let Some(opt) = a.take_any("opt") { opt } else { a.try_into()? } })
	}
}

impl FromLua for SearchForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for SearchForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
