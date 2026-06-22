use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{FromLua, Lua, UserData, UserDataFields, Value};
use serde::Deserialize;
use yazi_shared::{Layer, event::ActionCow};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Mixable, keymap::Chord};

#[repr(transparent)]
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ChordArc(Arc<Chord>);

impl Deref for ChordArc {
	type Target = Arc<Chord>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for ChordArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl AsRef<Chord> for ChordArc {
	fn as_ref(&self) -> &Chord { self }
}

impl From<&Self> for ChordArc {
	fn from(value: &Self) -> Self { value.clone() }
}

impl From<Chord> for ChordArc {
	fn from(value: Chord) -> Self { Self(Arc::new(value)) }
}

impl From<ChordArc> for Chord {
	fn from(value: ChordArc) -> Self {
		match Arc::try_unwrap(value.0) {
			Ok(c) => c,
			Err(arc) => Self::clone(&arc),
		}
	}
}

impl From<&ChordArc> for Chord {
	fn from(value: &ChordArc) -> Self { Self::clone(value) }
}

impl ChordArc {
	pub fn to_seq(&self, layer: Layer) -> Vec<ActionCow> {
		self
			.run
			.iter()
			.rev()
			.cloned()
			.map(|mut a| {
				a.layer = a.layer.or(layer);
				a.into()
			})
			.collect()
	}

	pub fn into_seq(self, layer: Layer) -> Vec<ActionCow> {
		match Arc::try_unwrap(self.0) {
			Ok(c) => c
				.run
				.into_iter()
				.rev()
				.map(|mut a| {
					a.layer = a.layer.or(layer);
					a.into()
				})
				.collect(),
			Err(arc) => Self(arc).to_seq(layer),
		}
	}
}

impl Mixable for ChordArc {
	fn filter(&self) -> bool { self.0.filter() }
}

impl FromLua for ChordArc {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::UserData(ud) => ud.take()?,
			_ => Chord::deserialize(mlua::serde::Deserializer::new(value))?.into(),
		})
	}
}

impl UserData for ChordArc {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));

		fields.add_cached_field("on", |lua, me| lua.create_sequence_from(me.on.iter().copied()));

		fields.add_cached_field("run", |_, me| Ok(me.run.clone()));

		fields.add_cached_field("desc", |lua, me| lua.create_string(&me.desc));
	}
}
