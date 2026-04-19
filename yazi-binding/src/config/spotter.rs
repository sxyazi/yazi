use std::{ops::Deref, sync::Arc};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, UserData, UserDataFields, Value};
use yazi_config::YAZI;

use crate::{FileRef, Id, Iter, cached_field};

#[derive(Clone)]
pub struct Spotter {
	inner: Arc<yazi_config::plugin::Spotter>,

	v_name: Option<Value>,
}

impl Deref for Spotter {
	type Target = yazi_config::plugin::Spotter;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Spotter {
	pub fn new(inner: impl Into<Arc<yazi_config::plugin::Spotter>>) -> Self {
		Self { inner: inner.into(), v_name: None }
	}
}

impl UserData for Spotter {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(Id(me.id)));

		cached_field!(fields, name, |lua, me| lua.create_string(&*me.name));
	}
}

// --- Matcher
pub struct SpotterMatcher(pub(super) yazi_config::plugin::SpotterMatcher<'static>);

impl SpotterMatcher {
	pub fn new(inner: impl Into<yazi_config::plugin::SpotterMatcher<'static>>) -> Self {
		Self(inner.into())
	}
}

impl TryFrom<Table> for SpotterMatcher {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();
		let file: Option<FileRef> = value.raw_get("file")?;
		let mime: Option<String> = value.raw_get("mime")?;

		Ok(Self(yazi_config::plugin::SpotterMatcher {
			spotters: YAZI.plugin.spotters.load_full(),
			id: id.0,
			file: file.map(|f| f.inner.clone().into()),
			mime: mime.map(Into::into),
			..Default::default()
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
