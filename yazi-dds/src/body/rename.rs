use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::{Id, url::Url};

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyRename<'a> {
	pub tab:  Id,
	pub from: Cow<'a, Url>,
	pub to:   Cow<'a, Url>,
}

impl<'a> BodyRename<'a> {
	#[inline]
	pub fn borrowed(tab: Id, from: &'a Url, to: &'a Url) -> Body<'a> {
		Self { tab, from: Cow::Borrowed(from), to: Cow::Borrowed(to) }.into()
	}
}

impl BodyRename<'static> {
	#[inline]
	pub fn dummy(tab: Id, from: &Url, to: &Url) -> Body<'static> {
		Self { tab, from: Cow::Owned(from.clone()), to: Cow::Owned(to.clone()) }.into()
	}
}

impl<'a> From<BodyRename<'a>> for Body<'a> {
	fn from(value: BodyRename<'a>) -> Self { Self::Rename(value) }
}

impl IntoLua for BodyRename<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.get().into_lua(lua)?),
				("from", yazi_binding::Url::new(self.from.into_owned()).into_lua(lua)?),
				("to", yazi_binding::Url::new(self.to.into_owned()).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
