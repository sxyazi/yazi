use hashbrown::HashMap;
use mlua::{IntoLua, Lua, Value};
use yazi_dds::Sendable;
use yazi_shared::{Id, SStr, data::{Data, DataKey}};

#[derive(Clone, Debug, Default)]
pub struct EntryJob {
	pub id:     Id,
	pub args:   HashMap<DataKey, Data>,
	pub plugin: SStr,
}

impl IntoLua for EntryJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("id", self.id.get().into_lua(lua)?),
				("args", Sendable::args_to_table(lua, self.args)?.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
