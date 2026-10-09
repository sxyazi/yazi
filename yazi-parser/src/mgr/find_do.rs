use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::mgr::FindDoOpt;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Clone, Debug)]
pub struct FindDoForm {
	pub opt: FindDoOpt,
}

impl<C> FromAction<C> for FindDoForm {
	fn from_action(mut a: ActionCow, cx: &C) -> anyhow::Result<Self> {
		Ok(Self {
			opt: if let Some(opt) = a.take_any("opt") { opt } else { FromAction::from_action(a, cx)? },
		})
	}
}

impl FromLua for FindDoForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for FindDoForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
