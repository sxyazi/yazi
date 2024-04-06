use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::fs::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyMove<'a> {
	pub from: Cow<'a, Url>,
	pub to: Cow<'a, Url>,
}

impl<'a> BodyMove<'a> {
	#[inline]
	pub fn borrowed(from: &'a Url, to: &'a Url) -> Body<'a> {
		Self { from: Cow::Borrowed(from), to: Cow::Borrowed(to) }.into()
	}
}

impl BodyMove<'static> {
	#[inline]
	pub fn dummy(from: &Url, to: &Url) -> Body<'static> {
		Self { from: Cow::Owned(from.clone()), to: Cow::Owned(to.clone()) }.into()
	}
}

impl<'a> From<BodyMove<'a>> for Body<'a> {
	fn from(value: BodyMove<'a>) -> Self {
		Self::Move(value)
	}
}

impl IntoLua<'_> for BodyMove<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("from", lua.create_any_userdata(self.from.into_owned())?.into_lua(lua)?),
				("to", lua.create_any_userdata(self.to.into_owned())?.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
