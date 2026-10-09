use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug)]
pub struct RemoveForm {
	pub force:       bool,
	pub permanently: bool,
	pub hovered:     bool,
}

impl<C> FromAction<C> for RemoveForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self {
			force:       a.bool("force"),
			permanently: a.bool("permanently"),
			hovered:     a.bool("hovered"),
		})
	}
}

impl FromLua for RemoveForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RemoveForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
