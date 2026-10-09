use mlua::{FromLua, IntoLua, Lua, Table, Value};
use yazi_fs::file::Files;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug, Default)]
pub struct WatchForm {
	pub files: Files,
}

impl<C> FromAction<C> for WatchForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self { files: a.take("files").unwrap_or_default() })
	}
}

impl From<Files> for WatchForm {
	fn from(files: Files) -> Self { Self { files } }
}

impl FromLua for WatchForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self { files: t.raw_get("files")? })
	}
}

impl IntoLua for WatchForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("files", self.files)])?.into_lua(lua)
	}
}
