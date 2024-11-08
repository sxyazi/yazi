use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::fs::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyTrash<'a> {
	pub urls: Cow<'a, Vec<Url>>,
}

impl<'a> BodyTrash<'a> {
	#[inline]
	pub fn borrowed(urls: &'a Vec<Url>) -> Body<'a> { Self { urls: Cow::Borrowed(urls) }.into() }
}

impl BodyTrash<'static> {
	#[inline]
	pub fn owned(urls: Vec<Url>) -> Body<'static> { Self { urls: Cow::Owned(urls) }.into() }
}

impl<'a> From<BodyTrash<'a>> for Body<'a> {
	fn from(value: BodyTrash<'a>) -> Self { Self::Trash(value) }
}

impl IntoLua for BodyTrash<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let urls = lua.create_table_with_capacity(self.urls.len(), 0)?;

		// In most cases, `self.urls` will be `Cow::Owned`, so
		// `.into_owned().into_iter()` can avoid any cloning, whereas
		// `.iter().cloned()` will clone each element.
		#[allow(clippy::unnecessary_to_owned)]
		for (i, url) in self.urls.into_owned().into_iter().enumerate() {
			urls.raw_set(i + 1, lua.create_any_userdata(url)?)?;
		}

		lua.create_table_from([("urls", urls)])?.into_lua(lua)
	}
}
