use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::Url;

use super::Ember;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmberDelete<'a> {
	pub urls: Cow<'a, Vec<Url>>,
}

impl<'a> EmberDelete<'a> {
	pub fn borrowed(urls: &'a Vec<Url>) -> Ember<'a> { Self { urls: Cow::Borrowed(urls) }.into() }
}

impl EmberDelete<'static> {
	pub fn owned(urls: Vec<Url>) -> Ember<'static> { Self { urls: Cow::Owned(urls) }.into() }
}

impl<'a> From<EmberDelete<'a>> for Ember<'a> {
	fn from(value: EmberDelete<'a>) -> Self { Self::Delete(value) }
}

impl IntoLua for EmberDelete<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let urls =
			lua.create_sequence_from(self.urls.into_owned().into_iter().map(yazi_binding::Url::new))?;

		lua.create_table_from([("urls", urls)])?.into_lua(lua)
	}
}
