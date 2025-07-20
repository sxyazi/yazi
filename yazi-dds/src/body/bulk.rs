use std::{borrow::Cow, collections::HashMap};

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyBulk<'a> {
	pub changes: HashMap<Cow<'a, Url>, Cow<'a, Url>>,
}

impl<'a> BodyBulk<'a> {
	pub fn borrowed<I>(changes: I) -> Body<'a>
	where
		I: Iterator<Item = (&'a Url, &'a Url)>,
	{
		Self { changes: changes.map(|(from, to)| (from.into(), to.into())).collect() }.into()
	}
}

impl BodyBulk<'static> {
	pub fn owned<'a, I>(changes: I) -> Body<'static>
	where
		I: Iterator<Item = (&'a Url, &'a Url)>,
	{
		Self { changes: changes.map(|(from, to)| (from.clone().into(), to.clone().into())).collect() }
			.into()
	}
}

impl<'a> From<BodyBulk<'a>> for Body<'a> {
	fn from(value: BodyBulk<'a>) -> Self { Self::Bulk(value) }
}

impl IntoLua for BodyBulk<'_> {
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
