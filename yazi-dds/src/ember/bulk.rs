use std::borrow::Cow;

use hashbrown::HashMap;
use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::UrlBuf;

use super::Ember;

#[derive(Debug, Deserialize, Serialize)]
pub struct EmberBulk<'a> {
	pub changes: HashMap<Cow<'a, UrlBuf>, Cow<'a, UrlBuf>>,
}

impl<'a> EmberBulk<'a> {
	pub fn borrowed<I>(changes: I) -> Ember<'a>
	where
		I: Iterator<Item = (&'a UrlBuf, &'a UrlBuf)>,
	{
		Self { changes: changes.map(|(from, to)| (from.into(), to.into())).collect() }.into()
	}
}

impl EmberBulk<'static> {
	pub fn owned<'a, I>(changes: I) -> Ember<'static>
	where
		I: Iterator<Item = (&'a UrlBuf, &'a UrlBuf)>,
	{
		Self { changes: changes.map(|(from, to)| (from.clone().into(), to.clone().into())).collect() }
			.into()
	}
}

impl<'a> From<EmberBulk<'a>> for Ember<'a> {
	fn from(value: EmberBulk<'a>) -> Self { Self::Bulk(value) }
}

impl IntoLua for EmberBulk<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from(
				self
					.changes
					.into_iter()
					.map(|(from, to)| (yazi_binding::Url::new(from), yazi_binding::Url::new(to))),
			)?
			.into_lua(lua)
	}
}
