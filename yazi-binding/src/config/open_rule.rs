use std::{ops::Deref, sync::Arc};

use mlua::{ExternalError, FromLua, IntoLua, Lua, LuaSerdeExt, Table, UserData, UserDataFields, Value};
use yazi_config::YAZI;

use crate::{FileRef, Id, Iter, cached_field};

pub struct OpenRule {
	inner: Arc<yazi_config::open::OpenRule>,

	v_use: Option<Value>,
}

impl Deref for OpenRule {
	type Target = Arc<yazi_config::open::OpenRule>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<OpenRule> for Arc<yazi_config::open::OpenRule> {
	fn from(value: OpenRule) -> Self { value.inner }
}

impl OpenRule {
	pub fn new(inner: impl Into<Arc<yazi_config::open::OpenRule>>) -> Self {
		Self { inner: inner.into(), v_use: None }
	}
}

impl FromLua for OpenRule {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self::new(lua.from_value::<yazi_config::open::OpenRule>(value)?))
	}
}

impl UserData for OpenRule {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(Id(me.id)));
		cached_field!(fields, use, |lua, me| {
			lua.create_sequence_from(me.r#use.iter().map(|s| s.as_str()))
		});
	}
}

// --- Matcher
pub struct OpenRuleMatcher(pub(super) yazi_config::open::OpenRuleMatcher<'static>);

impl OpenRuleMatcher {
	pub fn new(inner: impl Into<yazi_config::open::OpenRuleMatcher<'static>>) -> Self {
		Self(inner.into())
	}
}

impl TryFrom<Table> for OpenRuleMatcher {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();
		let file: Option<FileRef> = value.raw_get("file")?;
		let mime: Option<String> = value.raw_get("mime")?;

		Ok(Self(yazi_config::open::OpenRuleMatcher {
			rules: YAZI.open.load_full(),
			id: id.0,
			file: file.map(|f| f.inner.clone().into()),
			mime: mime.map(Into::into),
			..Default::default()
		}))
	}
}

impl FromLua for OpenRuleMatcher {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => return Err("expected a table of OpenRuleMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for OpenRuleMatcher {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Iter::new(self.0.into_iter().map(OpenRule::new), None).into_lua(lua)
	}
}
