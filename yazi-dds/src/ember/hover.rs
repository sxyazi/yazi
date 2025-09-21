use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::{Id, url::UrlBuf};

use super::Ember;

#[derive(Debug, Deserialize, Serialize)]
pub struct EmberHover<'a> {
	pub tab: Id,
	pub url: Option<Cow<'a, UrlBuf>>,
}

impl<'a> EmberHover<'a> {
	pub fn borrowed(tab: Id, url: Option<&'a UrlBuf>) -> Ember<'a> {
		Self { tab, url: url.map(Into::into) }.into()
	}
}

impl EmberHover<'static> {
	pub fn owned(tab: Id, _: Option<&UrlBuf>) -> Ember<'static> { Self { tab, url: None }.into() }
}

impl<'a> From<EmberHover<'a>> for Ember<'a> {
	fn from(value: EmberHover<'a>) -> Self { Self::Hover(value) }
}

impl IntoLua for EmberHover<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.get().into_lua(lua)?),
				("url", self.url.map(yazi_binding::Url::new).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
