use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::{Url, UrlCow};

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberBulkCreate<'a> {
	urls: Vec<UrlCow<'a>>,
}

impl<'a> EmberBulkCreate<'a> {
	pub(crate) fn borrowed<I>(urls: I) -> Ember<'a>
	where
		I: Iterator<Item = Url<'a>>,
	{
		Self { urls: urls.map(Into::into).collect() }.into()
	}
}

impl EmberBulkCreate<'static> {
	pub(crate) fn owned<'a, I>(urls: I) -> Ember<'static>
	where
		I: Iterator<Item = Url<'a>>,
	{
		Self { urls: urls.map(|url| url.to_owned().into()).collect() }.into()
	}
}

impl<'a> From<EmberBulkCreate<'a>> for Ember<'a> {
	fn from(value: EmberBulkCreate<'a>) -> Self { Self::BulkCreate(value) }
}

impl IntoLua for EmberBulkCreate<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let urls = lua.create_sequence_from(self.urls.into_iter().map(UrlCow::into_owned))?;

		lua.create_table_from([("urls", urls)])?.into_lua(lua)
	}
}
