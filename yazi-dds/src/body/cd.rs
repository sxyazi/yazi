use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::{Id, url::Url};

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyCd<'a> {
	pub tab: Id,
	pub url: Cow<'a, Url>,
	#[serde(skip)]
	dummy:   bool,
}

impl<'a> BodyCd<'a> {
	pub fn borrowed(tab: Id, url: &'a Url) -> Body<'a> {
		Self { tab, url: url.into(), dummy: false }.into()
	}
}

impl BodyCd<'static> {
	pub fn owned(tab: Id, _: &Url) -> Body<'static> {
		Self { tab, url: Default::default(), dummy: true }.into()
	}
}

impl<'a> From<BodyCd<'a>> for Body<'a> {
	fn from(value: BodyCd<'a>) -> Self { Self::Cd(value) }
}

impl IntoLua for BodyCd<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.get().into_lua(lua)?),
				("url", Some(self.url).filter(|_| !self.dummy).map(yazi_binding::Url::new).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
