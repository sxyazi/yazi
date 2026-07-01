use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlBuf};

#[derive(Debug)]
pub struct RemoveDoForm {
	pub permanently: bool,
	pub targets:     Vec<UrlBuf>,
}

impl From<ActionCow> for RemoveDoForm {
	fn from(mut a: ActionCow) -> Self {
		Self {
			permanently: a.bool("permanently"),
			targets:     a.take_any("targets").unwrap_or_default(),
		}
	}
}

impl FromLua for RemoveDoForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RemoveDoForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
