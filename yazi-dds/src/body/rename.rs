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
	pub fn borrowed(tab: Id, from: &'a Url, to: &'a Url) -> Body<'a> {
		Self { tab, from: from.into(), to: to.into() }.into()
	}
}

impl BodyRename<'static> {
	pub fn owned(tab: Id, from: &Url, to: &Url) -> Body<'static> {
		Self { tab, from: from.clone().into(), to: to.clone().into() }.into()
	}
}

impl<'a> From<BodyRename<'a>> for Body<'a> {
	fn from(value: BodyRename<'a>) -> Self { Self::Rename(value) }
}

impl IntoLua for BodyRename<'_> {
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
