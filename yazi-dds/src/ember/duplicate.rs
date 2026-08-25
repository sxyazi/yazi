use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::UrlBuf;

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberDuplicate<'a> {
	items: Cow<'a, Vec<EmberDuplicateItem>>,
}

impl<'a> EmberDuplicate<'a> {
	pub(crate) fn borrowed(items: &'a Vec<EmberDuplicateItem>) -> Ember<'a> {
		Self { items: Cow::Borrowed(items) }.into()
	}
}

impl EmberDuplicate<'static> {
	pub(crate) fn owned(items: Vec<EmberDuplicateItem>) -> Ember<'static> {
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
pub struct EmberDuplicateItem {
	pub(crate) from: UrlBuf,
	pub(crate) to:   UrlBuf,
}

impl IntoLua for EmberDuplicateItem {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("from", self.from), ("to", self.to)])?.into_lua(lua)
	}
}
