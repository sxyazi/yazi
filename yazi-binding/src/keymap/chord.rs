use std::ops::Deref;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, UserData, UserDataFields, Value};
use serde::Deserialize;
use yazi_codegen::FromLuaOwned;
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Id, Iter, event::Actions, keymap::Key};

#[derive(FromLuaOwned)]
pub struct Chord {
	inner: yazi_config::keymap::ChordArc,
}

impl Deref for Chord {
	type Target = yazi_config::keymap::ChordArc;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<&yazi_config::keymap::ChordArc> for Chord {
	fn from(value: &yazi_config::keymap::ChordArc) -> Self { Self { inner: value.clone() } }
}

impl From<Chord> for yazi_config::keymap::ChordArc {
	fn from(value: Chord) -> Self { value.inner }
}

impl Chord {
	pub fn new(inner: impl Into<yazi_config::keymap::ChordArc>) -> Self {
		Self { inner: inner.into() }
	}
}

impl TryFrom<(Value, yazi_shared::Layer)> for Chord {
	type Error = mlua::Error;

	fn try_from((value, layer): (Value, yazi_shared::Layer)) -> Result<Self, Self::Error> {
		use yazi_config::keymap::Chord as C;
		use yazi_shared::Layer as L;

		let de = mlua::serde::Deserializer::new(value);
		let inner = match layer {
			L::Null => C::<{ L::Null as u8 }>::deserialize(de)?.into(),
			L::App => C::<{ L::App as u8 }>::deserialize(de)?.into(),
			L::Mgr => C::<{ L::Mgr as u8 }>::deserialize(de)?.into(),
			L::Tasks => C::<{ L::Tasks as u8 }>::deserialize(de)?.into(),
			L::Spot => C::<{ L::Spot as u8 }>::deserialize(de)?.into(),
			L::Pick => C::<{ L::Pick as u8 }>::deserialize(de)?.into(),
			L::Input => C::<{ L::Input as u8 }>::deserialize(de)?.into(),
			L::Confirm => C::<{ L::Confirm as u8 }>::deserialize(de)?.into(),
			L::Help => C::<{ L::Help as u8 }>::deserialize(de)?.into(),
			L::Cmp => C::<{ L::Cmp as u8 }>::deserialize(de)?.into(),
			L::Which => C::<{ L::Which as u8 }>::deserialize(de)?.into(),
			L::Notify => C::<{ L::Notify as u8 }>::deserialize(de)?.into(),
		};

		Ok(Self { inner })
	}
}

impl UserData for Chord {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(Id(me.inner.id)));

		fields
			.add_cached_field("on", |lua, me| lua.create_sequence_from(me.on.iter().copied().map(Key)));

		fields.add_cached_field("run", |_, me| Ok(Actions::new(me.run.clone())));

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
