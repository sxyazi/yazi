use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::UrlBuf;

use super::Ember;

#[derive(Debug, Deserialize, Serialize)]
pub struct EmberDuplicate<'a> {
	pub items: Cow<'a, Vec<BodyDuplicateItem>>,
}

impl<'a> EmberDuplicate<'a> {
	pub fn borrowed(items: &'a Vec<BodyDuplicateItem>) -> Ember<'a> {
		Self { items: Cow::Borrowed(items) }.into()
	}
}

impl EmberDuplicate<'static> {
	pub fn owned(items: Vec<BodyDuplicateItem>) -> Ember<'static> {
		Self { items: Cow::Owned(items) }.into()
	}
}

impl<'a> From<EmberDuplicate<'a>> for Ember<'a> {
	fn from(value: EmberDuplicate<'a>) -> Self { Self::Duplicate(value) }
}

impl IntoLua for EmberDuplicate<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("items", self.items.into_owned())])?.into_lua(lua)
	}
}

// --- Item
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BodyDuplicateItem {
	pub from: UrlBuf,
	pub to:   UrlBuf,
}

impl IntoLua for BodyDuplicateItem {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("from", yazi_binding::Url::new(self.from)),
				("to", yazi_binding::Url::new(self.to)),
			])?
			.into_lua(lua)
	}
}
