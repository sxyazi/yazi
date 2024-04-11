use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::fs::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyDelete<'a> {
	pub urls: Cow<'a, Vec<Url>>,
}

impl<'a> BodyDelete<'a> {
	#[inline]
	pub fn borrowed(urls: &'a Vec<Url>) -> Body<'a> { Self { urls: Cow::Borrowed(urls) }.into() }
}

impl BodyDelete<'static> {
	#[inline]
	pub fn owned(urls: Vec<Url>) -> Body<'static> { Self { urls: Cow::Owned(urls) }.into() }
}

impl<'a> From<BodyDelete<'a>> for Body<'a> {
	fn from(value: BodyDelete<'a>) -> Self { Self::Delete(value) }
}

impl IntoLua<'_> for BodyDelete<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value<'_>> {
		let urls = lua.create_table_with_capacity(self.urls.len(), 0)?;
		for (i, url) in self.urls.into_owned().into_iter().enumerate() {
			urls.raw_set(i + 1, lua.create_any_userdata(url)?)?;
		}
		lua.create_table_from([("urls", urls)])?.into_lua(lua)
	}
}
