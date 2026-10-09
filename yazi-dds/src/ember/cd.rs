use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::{id::Id, url::UrlBuf};

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberCd<'a> {
	pub(super) tab: Id,
	url:            Cow<'a, UrlBuf>,
	#[serde(skip)]
	dummy:          bool,
}

impl<'a> EmberCd<'a> {
	pub(crate) fn borrowed(tab: Id, url: &'a UrlBuf) -> Ember<'a> {
		Self { tab, url: url.into(), dummy: false }.into()
	}
}

impl EmberCd<'static> {
	pub(crate) fn owned(tab: Id, _: &UrlBuf) -> Ember<'static> {
		Self { tab, url: Default::default(), dummy: true }.into()
	}
}

impl<'a> From<EmberCd<'a>> for Ember<'a> {
	fn from(value: EmberCd<'a>) -> Self { Self::Cd(value) }
}

impl IntoLua for EmberCd<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		if self.dummy {
			return lua.create_table()?.into_lua(lua);
		}

		lua
			.create_table_from([
				("tab", self.tab.into_lua(lua)?),
				("url", self.url.into_owned().into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
