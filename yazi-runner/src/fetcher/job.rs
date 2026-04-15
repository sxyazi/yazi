use std::sync::Arc;

use mlua::{IntoLua, Lua, Value};
use yazi_binding::File;
use yazi_dds::Sendable;

pub struct FetchJob {
	pub fetcher: Arc<yazi_config::plugin::Fetcher>,
	pub files:   Vec<yazi_fs::File>,
}

impl IntoLua for FetchJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("args", Sendable::args_to_table_ref(lua, &self.fetcher.args)?.into_lua(lua)?),
				("files", lua.create_sequence_from(self.files.into_iter().map(File::new))?.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
