use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::Url;

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

impl IntoLua for BodyMove<'static> {
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

impl IntoLua for BodyMoveItem {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("from", yazi_binding::Url::new(self.from)),
				("to", yazi_binding::Url::new(self.to)),
			])?
			.into_lua(lua)
	}
}
