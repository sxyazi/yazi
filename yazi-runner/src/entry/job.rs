use hashbrown::HashMap;
use mlua::{IntoLua, Lua, Value};
use yazi_shared::{data::{Data, DataKey}, id::Id, scope::{Scope, ScopeLike}, sendable::Sendable};
use yazi_shim::CompactString;

use crate::PluginJob;

#[derive(Clone, Debug, Default)]
pub struct EntryJob {
	pub id:     Id,
	pub args:   HashMap<DataKey, Data>,
	pub plugin: CompactString,
	pub scope:  Scope,
}

impl PluginJob for EntryJob {
	fn name(&self) -> &CompactString { &self.plugin }
}

impl ScopeLike for EntryJob {
	fn scope(&self) -> &Scope { &self.scope }
}

impl IntoLua for EntryJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("id", self.id.into_lua(lua)?),
				("scope", self.scope.into_lua(lua)?),
				("args", Sendable::args_to_table(lua, self.args)?.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
