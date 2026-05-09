use std::{ops::Deref, sync::Arc};

use mlua::{ExternalError, FromLua, IntoLua, Lua, LuaSerdeExt, Table, UserData, UserDataFields, Value};
use yazi_config::YAZI;

use crate::{FileRef, Id, Iter, cached_field};

#[derive(Clone)]
pub struct Previewer {
	inner: Arc<yazi_config::plugin::Previewer>,

	v_name: Option<Value>,
}

impl Deref for Previewer {
	type Target = yazi_config::plugin::Previewer;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<Previewer> for Arc<yazi_config::plugin::Previewer> {
	fn from(value: Previewer) -> Self { value.inner }
}

impl Previewer {
	pub fn new(inner: impl Into<Arc<yazi_config::plugin::Previewer>>) -> Self {
		Self { inner: inner.into(), v_name: None }
	}
}

impl FromLua for Previewer {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self::new(lua.from_value::<yazi_config::plugin::Previewer>(value)?))
	}
}

impl UserData for Previewer {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(Id(me.id)));

		cached_field!(fields, name, |lua, me| lua.create_string(&*me.name));
	}
}

// --- Matcher
pub struct PreviewerMatcher(pub(super) yazi_config::plugin::PreviewerMatcher<'static>);

impl PreviewerMatcher {
	pub fn new(inner: impl Into<yazi_config::plugin::PreviewerMatcher<'static>>) -> Self {
		Self(inner.into())
	}
}

impl TryFrom<Table> for PreviewerMatcher {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();
		let file: Option<FileRef> = value.raw_get("file")?;
		let mime: Option<String> = value.raw_get("mime")?;

		Ok(Self(yazi_config::plugin::PreviewerMatcher {
			previewers: YAZI.plugin.previewers.load_full(),
			id: id.0,
			file: file.map(|f| f.inner.clone().into()),
			mime: mime.map(Into::into),
			..Default::default()
		}))
	}
}

impl FromLua for PreviewerMatcher {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table of PreviewerMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for PreviewerMatcher {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Iter::new(self.0.map(Previewer::new), None).into_lua(lua)
	}
}
