use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::file::File;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug)]
pub struct ToggleForm {
	pub file:  Option<File>,
	pub state: Option<bool>,
}

impl<C> FromAction<C> for ToggleForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self {
			file:  a.take_first().ok(),
			state: match a.get("state") {
				Ok("on") => Some(true),
				Ok("off") => Some(false),
				_ => None,
			},
		})
	}
}

impl FromLua for ToggleForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ToggleForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
