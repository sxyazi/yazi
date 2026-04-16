use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlBuf};

#[derive(Debug)]
pub struct ToggleForm {
	pub url:   Option<UrlBuf>,
	pub state: Option<bool>,
}

impl From<ActionCow> for ToggleForm {
	fn from(mut a: ActionCow) -> Self {
		Self {
			url:   a.take_first().ok(),
			state: match a.get("state") {
				Ok("on") => Some(true),
				Ok("off") => Some(false),
				_ => None,
			},
		}
	}
}

impl FromLua for ToggleForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ToggleForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
