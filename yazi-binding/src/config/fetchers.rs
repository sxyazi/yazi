use mlua::{ExternalError, FromLua, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use yazi_config::YAZI;

use crate::{FileRef, Id, Iter, config::Fetcher};

pub struct Fetchers;

impl UserData for Fetchers {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |lua, _, matcher: Option<FetcherMatcher>| match matcher {
			Some(matcher) => matcher.into_lua(lua),
			None => FetcherMatcher::new(&YAZI.plugin.fetchers).into_lua(lua),
		});

		methods.add_meta_method(MetaMethod::Len, |_, _, ()| Ok(YAZI.plugin.fetchers.load().len()));
	}
}

// --- Matcher
pub struct FetcherMatcher(yazi_config::plugin::FetcherMatcher<'static>);

impl FetcherMatcher {
	fn new(inner: impl Into<yazi_config::plugin::FetcherMatcher<'static>>) -> Self {
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
			_ => return Err("expected a table of FetcherMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for FetcherMatcher {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Iter::new(self.0.into_iter().map(Fetcher::new), None).into_lua(lua)
	}
}
