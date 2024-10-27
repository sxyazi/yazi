use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyTab {
	pub idx: usize,
}

impl BodyTab {
	#[inline]
	pub fn owned(idx: usize) -> Body<'static> { Self { idx }.into() }
}

impl<'a> From<BodyTab> for Body<'a> {
	fn from(value: BodyTab) -> Self { Self::Tab(value) }
}

impl IntoLua for BodyTab {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("idx", self.idx)])?.into_lua(lua)
	}
}
