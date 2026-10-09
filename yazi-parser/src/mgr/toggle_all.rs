use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::file::File;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug)]
pub struct ToggleAllForm {
	pub files: Vec<File>,
	pub state: Option<bool>,
}

impl<C> FromAction<C> for ToggleAllForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self {
			files: a.take_seq(),
			state: match a.get("state") {
				Ok("on") => Some(true),
				Ok("off") => Some(false),
				_ => None,
			},
		})
	}
}

impl From<Option<bool>> for ToggleAllForm {
	fn from(state: Option<bool>) -> Self { Self { files: vec![], state } }
}

impl FromLua for ToggleAllForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ToggleAllForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
