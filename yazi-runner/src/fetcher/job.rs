use std::ops::Deref;

use mlua::{IntoLua, Lua, Value};
use yazi_binding::Scope;
use yazi_config::plugin::FetcherArc;
use yazi_fs::file::Files;
use yazi_shared::{id::Id, sendable::Sendable};
use yazi_shim::SStr;

use crate::PluginJob;

pub struct FetchJob {
	pub tab:     Id,
	pub fetcher: FetcherArc,
	pub files:   Files,
	pub scope:   Scope,
}

impl PluginJob for FetchJob {
	fn tab(&self) -> Id { self.tab }

	fn name(&self) -> &SStr { &self.fetcher.name }

	fn scope(&self) -> &Scope { &self.scope }
}

impl Deref for FetchJob {
	type Target = FetcherArc;

	fn deref(&self) -> &Self::Target { &self.fetcher }
}

impl IntoLua for FetchJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.into_lua(lua)?),
				("args", Sendable::args_to_table_ref(lua, &self.fetcher.args)?.into_lua(lua)?),
				("files", self.files.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
