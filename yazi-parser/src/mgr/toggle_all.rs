use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::CmdCow, url::UrlCow};

#[derive(Debug)]
pub struct ToggleAllOpt {
	pub urls:  Vec<UrlCow<'static>>,
	pub state: Option<bool>,
}

impl From<CmdCow> for ToggleAllOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			urls:  c.take_seq(),
			state: match c.get("state") {
				Ok("on") => Some(true),
				Ok("off") => Some(false),
				_ => None,
			},
		}
	}
}

impl From<Option<bool>> for ToggleAllOpt {
	fn from(state: Option<bool>) -> Self { Self { urls: vec![], state } }
}

impl FromLua for ToggleAllOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ToggleAllOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
