use std::{fmt, ops::{Deref, DerefMut}, ptr, sync::Arc};

use mlua::{FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use serde::Deserialize;
use yazi_macro::impl_data_any;
use yazi_shared::{Layer, event::ActionCow};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Mixable, keymap::{Chord, Chords}};

#[derive(Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct ChordArc {
	inner:            Arc<Chord>,
	#[serde(skip)]
	pub(super) owner: Option<&'static Chords>,
}

impl fmt::Debug for ChordArc {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("ChordArc")
			.field("inner", &self.inner)
			.field("owner", &self.owner.map(ptr::from_ref))
			.finish()
	}
}

impl_data_any!(ChordArc, from_into_lua = inherit);

impl Deref for ChordArc {
	type Target = Arc<Chord>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for ChordArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl AsRef<Chord> for ChordArc {
	fn as_ref(&self) -> &Chord { self }
}

impl From<&Self> for ChordArc {
	fn from(value: &Self) -> Self { value.clone() }
}

impl From<Chord> for ChordArc {
	fn from(value: Chord) -> Self { Self { inner: Arc::new(value), owner: None } }
}

impl From<ChordArc> for Chord {
	fn from(value: ChordArc) -> Self { Arc::unwrap_or_clone(value.inner) }
}

impl From<&ChordArc> for Chord {
	fn from(value: &ChordArc) -> Self { Self::clone(value) }
}

impl ChordArc {
	pub(super) fn clone_for(&self, owner: &'static Chords) -> Self {
		Self { inner: self.inner.clone(), owner: Some(owner) }
	}

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
		match Arc::try_unwrap(self.inner) {
			Ok(c) => c
				.run
				.into_iter()
				.rev()
				.map(|mut a| {
					a.layer = a.layer.or(layer);
					a.into()
				})
				.collect(),
			Err(inner) => Self { inner, owner: self.owner }.to_seq(layer),
		}
	}
}

impl Mixable for ChordArc {
	fn filter(&self) -> bool { self.inner.filter() }
}

impl FromLua for ChordArc {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::UserData(ud) => ud.borrow::<Self>()?.clone(),
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

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Close, |_, me, _: Value| {
			if let Some(owner) = me.owner {
				owner.remove(me.id.into());
			}
			Ok(())
		});
	}
}
