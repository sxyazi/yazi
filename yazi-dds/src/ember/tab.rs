use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::Id;

use super::Ember;

#[derive(Debug, Deserialize, Serialize)]
pub struct EmberTab {
	pub id: Id,
}

impl EmberTab {
	pub fn owned(id: Id) -> Ember<'static> { Self { id }.into() }

	pub fn borrowed(id: Id) -> Ember<'static> { Self::owned(id) }
}

impl From<EmberTab> for Ember<'_> {
	fn from(value: EmberTab) -> Self { Self::Tab(value) }
}

impl IntoLua for EmberTab {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("idx", self.id.get())])?.into_lua(lua)
	}
}
