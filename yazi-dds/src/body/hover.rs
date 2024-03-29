use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::fs::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyHover<'a> {
	pub owned: bool,
	pub tab:   usize,
	pub url:   Option<Cow<'a, Url>>,
}

impl<'a> BodyHover<'a> {
	#[inline]
	pub fn borrowed(tab: usize, url: Option<&'a Url>) -> Body<'a> {
		Self { owned: false, tab, url: url.map(Cow::Borrowed) }.into()
	}
}

impl BodyHover<'static> {
	#[inline]
	pub fn owned(tab: usize) -> Body<'static> { Self { owned: false, tab, url: None }.into() }
}

impl<'a> From<BodyHover<'a>> for Body<'a> {
	fn from(value: BodyHover<'a>) -> Self { Self::Hover(value) }
}

impl IntoLua<'_> for BodyHover<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		if self.owned && self.url.is_some() {
			lua.create_table_from([
				("tab", self.tab.into_lua(lua)?),
				("url", lua.create_any_userdata(self.url.unwrap().into_owned())?.into_lua(lua)?),
			])?
		} else {
			lua.create_table_from([("tab", self.tab)])?
		}
		.into_lua(lua)
	}
}
