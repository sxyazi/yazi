use mlua::{ExternalError, FromLua, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use yazi_config::YAZI;

use crate::{FileRef, Id, Iter, config::Spotter};

pub struct Spotters;

impl UserData for Spotters {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |_, _, matcher: SpotterMatcher| Ok(matcher));

		methods.add_meta_method(MetaMethod::Len, |_, _, ()| Ok(YAZI.plugin.spotters.load().len()));
	}
}

// --- Matcher
pub struct SpotterMatcher(yazi_config::plugin::SpotterMatcher<'static>);

impl TryFrom<Table> for SpotterMatcher {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();
		let file: Option<FileRef> = value.raw_get("file")?;
		let mime: Option<String> = value.raw_get("mime")?;

		Ok(Self(yazi_config::plugin::SpotterMatcher {
			spotters: YAZI.plugin.spotters.load_full(),
			id:       id.0,
			file:     file.map(|f| f.inner.clone().into()),
			mime:     mime.map(Into::into),
			offset:   0,
		}))
	}
}

impl FromLua for SpotterMatcher {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => return Err("expected a table of SpotterMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for SpotterMatcher {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Iter::new(self.0.into_iter().map(Spotter::new), None).into_lua(lua)
	}
}
