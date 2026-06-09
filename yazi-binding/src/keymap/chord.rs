use std::{ops::Deref, sync::Arc};

use mlua::{ExternalError, FromLua, IntoLua, Lua, LuaSerdeExt, Table, UserData, UserDataFields, Value};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Id, Iter, event::Action, keymap::Key};

pub struct Chord {
	inner: Arc<yazi_config::keymap::Chord>,
}

impl Deref for Chord {
	type Target = Arc<yazi_config::keymap::Chord>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<Chord> for Arc<yazi_config::keymap::Chord> {
	fn from(value: Chord) -> Self { value.inner }
}

impl Chord {
	pub fn new(inner: impl Into<Arc<yazi_config::keymap::Chord>>) -> Self {
		Self { inner: inner.into() }
	}
}

impl FromLua for Chord {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self::new(lua.from_value::<yazi_config::keymap::Chord>(value)?))
	}
}

impl UserData for Chord {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(Id(me.inner.id)));

		fields
			.add_cached_field("on", |lua, me| lua.create_sequence_from(me.on.iter().copied().map(Key)));

		fields.add_cached_field("run", |lua, me| {
			lua.create_sequence_from(me.run.iter().cloned().map(Action::new))
		});

		fields.add_cached_field("desc", |lua, me| lua.create_string(&me.desc));
	}
}

// --- Matcher
pub struct ChordMatcher(pub(super) yazi_config::keymap::ChordMatcher);

impl TryFrom<Table> for ChordMatcher {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();

		Ok(Self(yazi_config::keymap::ChordMatcher { id: id.0, ..Default::default() }))
	}
}

impl FromLua for ChordMatcher {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table of ChordMatcher".into_lua_err()),
		}
	}
}

// --- ChordIter
pub struct ChordIter(pub(super) yazi_config::keymap::ChordIter);

impl ChordIter {
	pub fn new(inner: impl Into<yazi_config::keymap::ChordIter>) -> Self { Self(inner.into()) }
}

impl IntoLua for ChordIter {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Iter::new(self.0.map(Chord::new), None).into_lua(lua)
	}
}
