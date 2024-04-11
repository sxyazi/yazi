use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::fs::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyMove<'a> {
	pub items: Cow<'a, Vec<BodyMoveItem>>,
}

impl<'a> BodyMove<'a> {
	#[inline]
	pub fn borrowed(items: &'a Vec<BodyMoveItem>) -> Body<'a> {
		Self { items: Cow::Borrowed(items) }.into()
	}
}

impl BodyMove<'static> {
	#[inline]
	pub fn owned(items: Vec<BodyMoveItem>) -> Body<'static> {
		Self { items: Cow::Owned(items) }.into()
	}
}

impl<'a> From<BodyMove<'a>> for Body<'a> {
	fn from(value: BodyMove<'a>) -> Self { Self::Move(value) }
}

impl IntoLua<'_> for BodyMove<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("items", self.items.into_owned())])?.into_lua(lua)
	}
}

// --- Item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyMoveItem {
	pub from: Url,
	pub to:   Url,
}

impl IntoLua<'_> for BodyMoveItem {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("from", lua.create_any_userdata(self.from)?),
				("to", lua.create_any_userdata(self.to)?),
			])?
			.into_lua(lua)
	}
}
