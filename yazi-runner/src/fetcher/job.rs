use std::ops::Deref;

use mlua::{IntoLua, Lua, Value};
use yazi_config::plugin::FetcherArc;
use yazi_fs::file::Files;
use yazi_shared::data::Sendable;

pub struct FetchJob {
	pub fetcher: FetcherArc,
	pub files:   Files,
}

impl Deref for FetchJob {
	type Target = FetcherArc;

	fn deref(&self) -> &Self::Target { &self.fetcher }
}

impl IntoLua for FetchJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("args", Sendable::args_to_table_ref(lua, &self.fetcher.args)?.into_lua(lua)?),
				("files", self.files.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
