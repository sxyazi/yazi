use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlBuf};

#[derive(Debug)]
pub struct RemoveForm {
	pub force:       bool,
	pub permanently: bool,
	pub hovered:     bool,
	pub targets:     Vec<UrlBuf>,
}

impl From<ActionCow> for RemoveForm {
	fn from(mut a: ActionCow) -> Self {
		Self {
			force:       a.bool("force"),
			permanently: a.bool("permanently"),
			hovered:     a.bool("hovered"),
			targets:     a.take_any("targets").unwrap_or_default(),
		}
	}
}

impl FromLua for RemoveForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RemoveForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
