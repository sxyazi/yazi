use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlCow};

#[derive(Debug)]
pub struct ToggleAllOpt {
	pub urls:  Vec<UrlCow<'static>>,
	pub state: Option<bool>,
}

impl From<ActionCow> for ToggleAllOpt {
	fn from(mut a: ActionCow) -> Self {
		Self {
			urls:  a.take_seq(),
			state: match a.get("state") {
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
