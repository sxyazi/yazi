use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::CmdCow, url::UrlCow};

#[derive(Debug)]
pub struct ToggleOpt {
	pub url:   Option<UrlCow<'static>>,
	pub state: Option<bool>,
}

impl From<CmdCow> for ToggleOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			url:   c.take_first().ok(),
			state: match c.get("state") {
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
