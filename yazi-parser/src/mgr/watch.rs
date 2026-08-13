use mlua::{FromLua, IntoLua, Lua, Value};
use yazi_fs::file::File;
use yazi_shared::event::ActionCow;

#[derive(Debug, Default)]
pub struct WatchForm {
	pub files: Vec<File>,
}

impl From<ActionCow> for WatchForm {
	fn from(mut a: ActionCow) -> Self { Self { files: a.take_seq() } }
}

impl From<Vec<File>> for WatchForm {
	fn from(files: Vec<File>) -> Self { Self { files } }
}

impl FromLua for WatchForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self { files: Vec::from_lua(value, lua)? })
	}
}

impl IntoLua for WatchForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.files.into_lua(lua) }
}
