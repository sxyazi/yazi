use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::{Id, url::UrlBuf};

use super::Ember;

#[derive(Debug, Deserialize, Serialize)]
pub struct EmberRename<'a> {
	pub tab:  Id,
	pub from: Cow<'a, UrlBuf>,
	pub to:   Cow<'a, UrlBuf>,
}

impl<'a> EmberRename<'a> {
	pub fn borrowed(tab: Id, from: &'a UrlBuf, to: &'a UrlBuf) -> Ember<'a> {
		Self { tab, from: from.into(), to: to.into() }.into()
	}
}

impl EmberRename<'static> {
	pub fn owned(tab: Id, from: &UrlBuf, to: &UrlBuf) -> Ember<'static> {
		Self { tab, from: from.clone().into(), to: to.clone().into() }.into()
	}
}

impl<'a> From<EmberRename<'a>> for Ember<'a> {
	fn from(value: EmberRename<'a>) -> Self { Self::Rename(value) }
}

impl IntoLua for EmberRename<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.get().into_lua(lua)?),
				("from", yazi_binding::Url::new(self.from).into_lua(lua)?),
				("to", yazi_binding::Url::new(self.to).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
