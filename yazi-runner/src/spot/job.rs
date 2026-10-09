use mlua::{IntoLua, Lua, Value};
use yazi_config::plugin::SpotterArc;
use yazi_fs::file::File;
use yazi_shared::{id::{Id, Ids}, pool::Symbol, sendable::Sendable};
use yazi_shim::SStr;

use crate::PluginJob;

static IDS: Ids = Ids::new();

#[derive(Clone, Debug)]
pub struct SpotJob {
	pub tab:     Id,
	pub spotter: SpotterArc,
	pub file:    File,
	pub mime:    Symbol<str>,
	pub skip:    usize,
}

impl PluginJob for SpotJob {
	fn tab(&self) -> Id { self.tab }

	fn name(&self) -> &SStr { &self.spotter.name }
}

impl IntoLua for SpotJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("id", IDS.next().into_lua(lua)?),
				("args", Sendable::args_to_table_ref(lua, &self.spotter.args)?.into_lua(lua)?),
				("file", self.file.into_lua(lua)?),
				("mime", self.mime.into_lua(lua)?),
				("skip", self.skip.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
