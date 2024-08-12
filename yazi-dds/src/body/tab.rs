use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyTabSwitch {
	/// The index of the tab
	pub tab: usize,
}

impl<'a> BodyTabSwitch {
	#[inline]
	pub fn borrowed(tab: usize) -> Body<'a> { Self { tab }.into() }
}

impl BodyTabSwitch {
	#[inline]
	pub fn dummy(tab: usize) -> Body<'static> { Self { tab }.into() }
}

impl<'a> From<BodyTabSwitch> for Body<'a> {
	fn from(value: BodyTabSwitch) -> Self { Self::TabSwitch(value) }
}

impl IntoLua<'_> for BodyTabSwitch {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("tab", self.tab)])?.into_lua(lua)
	}
}
