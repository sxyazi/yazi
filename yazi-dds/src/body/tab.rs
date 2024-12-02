use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::Id;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyTab {
	pub id: Id,
}

impl BodyTab {
	#[inline]
	pub fn owned(id: Id) -> Body<'static> { Self { id }.into() }
}

impl From<BodyTab> for Body<'_> {
	fn from(value: BodyTab) -> Self { Self::Tab(value) }
}

impl IntoLua for BodyTab {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("idx", self.id.get())])?.into_lua(lua)
	}
}
