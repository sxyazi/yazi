use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct HiddenOpt {
	pub state: Option<bool>,
}

impl From<CmdCow> for HiddenOpt {
	fn from(c: CmdCow) -> Self {
		let state = match c.first_str() {
			Some("show") => Some(true),
			Some("hide") => Some(false),
			_ => None,
		};

		Self { state }
	}
}

impl IntoLua for &HiddenOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
