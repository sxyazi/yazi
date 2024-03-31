use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::fs::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyCd<'a> {
	pub tab: usize,
	pub url: Cow<'a, Url>,
	#[serde(skip)]
	dummy:   bool,
}

impl<'a> BodyCd<'a> {
	#[inline]
	pub fn borrowed(tab: usize, url: &'a Url) -> Body<'a> {
		Self { tab, url: Cow::Borrowed(url), dummy: false }.into()
	}
}

impl BodyCd<'static> {
	#[inline]
	pub fn dummy(tab: usize) -> Body<'static> {
		Self { tab, url: Default::default(), dummy: true }.into()
	}
}

impl<'a> From<BodyCd<'a>> for Body<'a> {
	fn from(value: BodyCd<'a>) -> Self { Self::Cd(value) }
}

impl IntoLua<'_> for BodyCd<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		if let Some(Cow::Owned(url)) = Some(self.url).filter(|_| !self.dummy) {
			lua.create_table_from([
				("tab", self.tab.into_lua(lua)?),
				("url", lua.create_any_userdata(url)?.into_lua(lua)?),
			])?
		} else {
			lua.create_table_from([("tab", self.tab)])?
		}
		.into_lua(lua)
	}
}
