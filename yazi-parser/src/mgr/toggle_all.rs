use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::CmdCow, url::UrlCow};

#[derive(Debug)]
pub struct ToggleAllOpt {
	pub urls:  Vec<UrlCow<'static>>,
	pub state: Option<bool>,
}

impl From<CmdCow> for ToggleAllOpt {
	fn from(mut c: CmdCow) -> Self {
		let mut urls = Vec::with_capacity(c.len());
		for i in 0..c.len() {
			match c.take_url(i) {
				Some(url) => urls.push(url),
				None => break,
			}
		}

		Self {
			urls,
			state: match c.str("state") {
				Some("on") => Some(true),
				Some("off") => Some(false),
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
