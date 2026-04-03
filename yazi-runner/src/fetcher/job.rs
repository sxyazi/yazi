use mlua::{IntoLua, Value};
use yazi_binding::File;
use yazi_dds::Sendable;
use yazi_shared::event::Action;

pub struct FetchJob {
	pub action: &'static Action,
	pub files:  Vec<yazi_fs::File>,
}

impl IntoLua for FetchJob {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("args", Sendable::args_to_table_ref(lua, &self.action.args)?.into_lua(lua)?),
				("files", lua.create_sequence_from(self.files.into_iter().map(File::new))?.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
