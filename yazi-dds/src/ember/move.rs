use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::UrlBuf;

use super::Ember;

#[derive(Debug, Deserialize, Serialize)]
pub struct EmberMove<'a> {
	pub items: Cow<'a, Vec<BodyMoveItem>>,
}

impl<'a> EmberMove<'a> {
	pub fn borrowed(items: &'a Vec<BodyMoveItem>) -> Ember<'a> {
		Self { items: Cow::Borrowed(items) }.into()
	}
}

impl EmberMove<'static> {
	pub fn owned(items: Vec<BodyMoveItem>) -> Ember<'static> {
		Self { items: Cow::Owned(items) }.into()
	}
}

impl<'a> From<EmberMove<'a>> for Ember<'a> {
	fn from(value: EmberMove<'a>) -> Self { Self::Move(value) }
}

impl IntoLua for EmberMove<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("items", self.items.into_owned())])?.into_lua(lua)
	}
}

// --- Item
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BodyMoveItem {
	pub from: UrlBuf,
	pub to:   UrlBuf,
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
