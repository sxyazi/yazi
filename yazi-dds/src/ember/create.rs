use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::{id::Id, url::{UrlBuf, UrlCow}};

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberCreate<'a> {
	pub(super) tab: Id,
	url:            UrlCow<'a>,
}

impl<'a> EmberCreate<'a> {
	pub(crate) fn borrowed(tab: Id, url: &'a UrlBuf) -> Ember<'a> {
		Self { tab, url: url.into() }.into()
	}
}

impl EmberCreate<'static> {
	pub(crate) fn owned(tab: Id, url: &UrlBuf) -> Ember<'static> {
		Self { tab, url: url.clone().into() }.into()
	}
}

impl<'a> From<EmberCreate<'a>> for Ember<'a> {
	fn from(value: EmberCreate<'a>) -> Self { Self::Create(value) }
}

impl IntoLua for EmberCreate<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.into_lua(lua)?),
				("url", self.url.into_owned().into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
