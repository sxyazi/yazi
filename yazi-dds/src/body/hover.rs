use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::{Id, url::Url};

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyHover<'a> {
	pub tab: Id,
	pub url: Option<Cow<'a, Url>>,
}

impl<'a> BodyHover<'a> {
	#[inline]
	pub fn borrowed(tab: Id, url: Option<&'a Url>) -> Body<'a> {
		Self { tab, url: url.map(Cow::Borrowed) }.into()
	}
}

impl BodyHover<'static> {
	#[inline]
	pub fn dummy(tab: Id) -> Body<'static> { Self { tab, url: None }.into() }
}

impl<'a> From<BodyHover<'a>> for Body<'a> {
	fn from(value: BodyHover<'a>) -> Self { Self::Hover(value) }
}

impl IntoLua for BodyHover<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		if let Some(Cow::Owned(url)) = self.url {
			lua.create_table_from([
				("tab", self.tab.get().into_lua(lua)?),
				("url", yazi_binding::Url::new(url).into_lua(lua)?),
			])?
		} else {
			lua.create_table_from([("tab", self.tab.get())])?
		}
		.into_lua(lua)
	}
}
