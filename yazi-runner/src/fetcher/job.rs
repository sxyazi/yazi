use std::ops::Deref;

use mlua::{IntoLua, Lua, Value};
use yazi_config::plugin::FetcherArc;
use yazi_fs::file::Files;
use yazi_shared::{scope::{Scope, ScopeLike}, sendable::Sendable};
use yazi_shim::SStr;

use crate::PluginJob;

pub struct FetchJob {
	pub fetcher: FetcherArc,
	pub files:   Files,
	pub scope:   Scope,
}

impl PluginJob for FetchJob {
	fn name(&self) -> &SStr { &self.fetcher.name }
}

impl ScopeLike for FetchJob {
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
				("scope", self.scope.into_lua(lua)?),
				("args", Sendable::args_to_table_ref(lua, &self.fetcher.args)?.into_lua(lua)?),
				("files", self.files.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
