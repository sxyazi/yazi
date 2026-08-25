use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::UrlBuf;

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberMove<'a> {
	items: Cow<'a, Vec<EmberMoveItem>>,
}

impl<'a> EmberMove<'a> {
	pub(crate) fn borrowed(items: &'a Vec<EmberMoveItem>) -> Ember<'a> {
		Self { items: Cow::Borrowed(items) }.into()
	}
}

impl EmberMove<'static> {
	pub(crate) fn owned(items: Vec<EmberMoveItem>) -> Ember<'static> {
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
pub struct EmberMoveItem {
	pub(crate) from: UrlBuf,
	pub(crate) to:   UrlBuf,
}

impl IntoLua for EmberMoveItem {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("from", self.from), ("to", self.to)])?.into_lua(lua)
	}
}
