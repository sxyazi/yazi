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
	pub fn borrowed(tab: Id, url: Option<&'a Url>) -> Body<'a> {
		Self { tab, url: url.map(Into::into) }.into()
	}
}

impl BodyHover<'static> {
	pub fn owned(tab: Id, _: Option<&Url>) -> Body<'static> { Self { tab, url: None }.into() }
}

impl<'a> From<BodyHover<'a>> for Body<'a> {
	fn from(value: BodyHover<'a>) -> Self { Self::Hover(value) }
}

impl IntoLua for BodyHover<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.get().into_lua(lua)?),
				("url", self.url.map(yazi_binding::Url::new).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
