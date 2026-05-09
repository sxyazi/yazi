use std::{ops::Deref, sync::Arc};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, UserData, UserDataFields, Value};
use yazi_config::YAZI;

use crate::{FileRef, Id, Iter, cached_field};

#[derive(Clone)]
pub struct Fetcher {
	inner: Arc<yazi_config::plugin::Fetcher>,

	v_name: Option<Value>,
}

impl Deref for Fetcher {
	type Target = yazi_config::plugin::Fetcher;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Fetcher {
	pub fn new(inner: impl Into<Arc<yazi_config::plugin::Fetcher>>) -> Self {
		Self { inner: inner.into(), v_name: None }
	}
}

impl UserData for Fetcher {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(Id(me.id)));

		cached_field!(fields, name, |lua, me| lua.create_string(&*me.name));
	}
}

// --- Matcher
pub struct FetcherMatcher(yazi_config::plugin::FetcherMatcher<'static>);

impl FetcherMatcher {
	pub fn new(inner: impl Into<yazi_config::plugin::FetcherMatcher<'static>>) -> Self {
		Self(inner.into())
	}
}

impl TryFrom<Table> for FetcherMatcher {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();
		let file: Option<FileRef> = value.raw_get("file")?;
		let mime: Option<String> = value.raw_get("mime")?;

		Ok(Self(yazi_config::plugin::FetcherMatcher {
			fetchers: YAZI.plugin.fetchers.load_full(),
			id: id.0,
			file: file.map(|f| f.inner.clone().into()),
			mime: mime.map(Into::into),
			..Default::default()
		}))
	}
}

impl FromLua for FetcherMatcher {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table of FetcherMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for FetcherMatcher {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Iter::new(self.0.map(Fetcher::new), None).into_lua(lua)
	}
}
