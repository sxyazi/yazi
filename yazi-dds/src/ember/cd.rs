use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::{Id, url::UrlBuf};

use super::Ember;

#[derive(Debug, Deserialize, Serialize)]
pub struct EmberCd<'a> {
	pub tab: Id,
	pub url: Cow<'a, UrlBuf>,
	#[serde(skip)]
	dummy:   bool,
}

impl<'a> EmberCd<'a> {
	pub fn borrowed(tab: Id, url: &'a UrlBuf) -> Ember<'a> {
		Self { tab, url: url.into(), dummy: false }.into()
	}
}

impl EmberCd<'static> {
	pub fn owned(tab: Id, _: &UrlBuf) -> Ember<'static> {
		Self { tab, url: Default::default(), dummy: true }.into()
	}
}

impl<'a> From<EmberCd<'a>> for Ember<'a> {
	fn from(value: EmberCd<'a>) -> Self { Self::Cd(value) }
}

impl IntoLua for EmberCd<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.get().into_lua(lua)?),
				("url", Some(self.url).filter(|_| !self.dummy).map(yazi_binding::Url::new).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
