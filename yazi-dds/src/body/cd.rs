use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::fs::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyCd<'a> {
	pub owned: bool,
	pub tab:   usize,
	pub url:   Cow<'a, Url>,
}

impl<'a> BodyCd<'a> {
	#[inline]
	pub fn borrowed(tab: usize, url: &'a Url) -> Body<'a> {
		Self { owned: false, tab, url: Cow::Borrowed(url) }.into()
	}
}

impl BodyCd<'static> {
	#[inline]
	pub fn owned(tab: usize) -> Body<'static> {
		Self { owned: false, tab, url: Default::default() }.into()
	}
}

impl<'a> From<BodyCd<'a>> for Body<'a> {
	fn from(value: BodyCd<'a>) -> Self { Self::Cd(value) }
}

impl IntoLua<'_> for BodyCd<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		if self.owned {
			lua.create_table_from([
				("tab", self.tab.into_lua(lua)?),
				("url", lua.create_any_userdata(self.url.into_owned())?.into_lua(lua)?),
			])?
		} else {
			lua.create_table_from([("tab", self.tab)])?
		}
		.into_lua(lua)
	}
}
