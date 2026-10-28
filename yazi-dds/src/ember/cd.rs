use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::{id::Id, url::UrlBuf};

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberCd<'a> {
	pub(super) tab: Id,
	url:            Cow<'a, UrlBuf>,
}

impl<'a> EmberCd<'a> {
	pub(crate) fn borrowed(tab: Id, url: &'a UrlBuf) -> Ember<'a> {
		Self { tab, url: url.into() }.into()
	}
}

impl EmberCd<'static> {
	pub(crate) fn owned(tab: Id, url: &UrlBuf) -> Ember<'static> {
		Self { tab, url: url.clone().into() }.into()
	}
}

impl<'a> From<EmberCd<'a>> for Ember<'a> {
	fn from(value: EmberCd<'a>) -> Self { Self::Cd(value) }
}

impl IntoLua for EmberCd<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.into_lua(lua)?),
				("url", self.url.into_owned().into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
