use std::{ops::Deref, sync::Arc};

use mlua::{ExternalError, FromLua, IntoLua, Lua, LuaSerdeExt, Table, UserData, UserDataFields, Value};

use crate::{Id, Iter, cached_field};

pub struct OpenerRule {
	inner: Arc<yazi_config::opener::OpenerRule>,

	v_run:  Option<Value>,
	v_desc: Option<Value>,
}

impl Deref for OpenerRule {
	type Target = Arc<yazi_config::opener::OpenerRule>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<OpenerRule> for Arc<yazi_config::opener::OpenerRule> {
	fn from(value: OpenerRule) -> Self { value.inner }
}

impl OpenerRule {
	pub fn new(inner: impl Into<Arc<yazi_config::opener::OpenerRule>>) -> Self {
		Self { inner: inner.into(), v_run: None, v_desc: None }
	}
}

impl FromLua for OpenerRule {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let mut inner: yazi_config::opener::OpenerRule = lua.from_value(value)?;
		inner.fill();

		Ok(Self::new(inner))
	}
}

impl UserData for OpenerRule {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(Id(me.id)));
		cached_field!(fields, run, |lua, me| lua.create_string(&*me.run));
		fields.add_field_method_get("block", |_, me| Ok(me.block));
		fields.add_field_method_get("orphan", |_, me| Ok(me.orphan));
		cached_field!(fields, desc, |lua, me| lua.create_string(&*me.desc));
	}
}

// --- Matcher
pub struct OpenerRuleMatcher(pub(super) yazi_config::opener::OpenerRuleMatcher);

impl OpenerRuleMatcher {
	pub fn new(inner: impl Into<yazi_config::opener::OpenerRuleMatcher>) -> Self {
		Self(inner.into())
	}
}

impl TryFrom<Table> for OpenerRuleMatcher {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let id: Id = t.raw_get("id").unwrap_or_default();

		Ok(Self(yazi_config::opener::OpenerRuleMatcher { id: id.0, ..Default::default() }))
	}
}

impl FromLua for OpenerRuleMatcher {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table of OpenerRuleMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for OpenerRuleMatcher {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Iter::new(self.0.into_iter().map(OpenerRule::new), None).into_lua(lua)
	}
}
