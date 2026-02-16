use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::UrlBuf;

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberDownload<'a> {
	pub urls: Cow<'a, Vec<UrlBuf>>,
}

impl<'a> EmberDownload<'a> {
	pub fn borrowed(urls: &'a Vec<UrlBuf>) -> Ember<'a> { Self { urls: Cow::Borrowed(urls) }.into() }
}

impl EmberDownload<'static> {
	pub fn owned(urls: Vec<UrlBuf>) -> Ember<'static> { Self { urls: Cow::Owned(urls) }.into() }
}

impl<'a> From<EmberDownload<'a>> for Ember<'a> {
	fn from(value: EmberDownload<'a>) -> Self { Self::Download(value) }
}

impl IntoLua for EmberDownload<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let urls =
			lua.create_sequence_from(self.urls.into_owned().into_iter().map(yazi_binding::Url::new))?;

		lua.create_table_from([("urls", urls)])?.into_lua(lua)
	}
}
