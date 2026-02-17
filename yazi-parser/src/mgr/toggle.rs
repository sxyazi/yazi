use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlCow};

#[derive(Debug)]
pub struct ToggleOpt {
	pub url:   Option<UrlCow<'static>>,
	pub state: Option<bool>,
}

impl From<ActionCow> for ToggleOpt {
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

impl FromLua for ToggleOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ToggleOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
