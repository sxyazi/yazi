use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::{ActionCow, FromAction}, url::UrlBuf};

#[derive(Debug)]
pub struct RemoveDoForm {
	pub permanently: bool,
	pub targets:     Vec<UrlBuf>,
}

impl<C> FromAction<C> for RemoveDoForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self {
			permanently: a.bool("permanently"),
			targets:     a.take_any("targets").unwrap_or_default(),
		})
	}
}

impl FromLua for RemoveDoForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RemoveDoForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
