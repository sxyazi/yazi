use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{UserData, UserDataFields, Value};
use serde::Deserialize;
use yazi_codegen::FromLuaOwned;
use yazi_shared::{Layer, event::ActionCow};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Mixable, keymap::Chord};

#[repr(transparent)]
#[derive(Clone, Debug, Default, Deserialize, FromLuaOwned)]
pub struct ChordArc<const L: u8 = { Layer::Null as u8 }>(Arc<Chord<L>>);

impl<const L: u8> Deref for ChordArc<L> {
	type Target = Arc<Chord<L>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<const L: u8> DerefMut for ChordArc<L> {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<const L: u8> From<&ChordArc<L>> for ChordArc<L> {
	fn from(value: &ChordArc<L>) -> Self { value.clone() }
}

impl<const L: u8, const M: u8> From<Chord<L>> for ChordArc<M> {
	fn from(value: Chord<L>) -> Self { ChordArc(Arc::new(value)).into_erased() }
}

impl<const L: u8> From<ChordArc<L>> for Chord<L> {
	fn from(value: ChordArc<L>) -> Self {
		match Arc::try_unwrap(value.0) {
			Ok(c) => c,
			Err(arc) => Self::clone(&arc),
		}
	}
}

impl<const L: u8> From<&ChordArc<L>> for Chord<L> {
	fn from(value: &ChordArc<L>) -> Self { Self::clone(value) }
}

impl TryFrom<(Value, Layer)> for ChordArc {
	type Error = mlua::Error;

	fn try_from((value, layer): (Value, Layer)) -> Result<Self, Self::Error> {
		use Chord as C;
		use Layer as L;

		let de = mlua::serde::Deserializer::new(value);
		Ok(match layer {
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
		})
	}
}

impl<const L: u8> ChordArc<L> {
	pub fn as_erased<const M: u8>(&self) -> &ChordArc<M> {
		unsafe { &*(self as *const ChordArc<L> as *const ChordArc<M>) }
	}

	pub fn into_erased<const M: u8>(self) -> ChordArc<M> {
		ChordArc(unsafe { Arc::from_raw(Arc::into_raw(self.0) as *const Chord<M>) })
	}

	pub fn to_seq(&self) -> Vec<ActionCow> {
		self.run.iter().rev().cloned().map(Into::into).collect()
	}

	pub fn into_seq(self) -> Vec<ActionCow> {
		match Arc::try_unwrap(self.0) {
			Ok(c) => c.run.into_iter().rev().map(Into::into).collect(),
			Err(arc) => Self(arc).to_seq(),
		}
	}
}

impl<const L: u8> Mixable for ChordArc<L> {
	fn filter(&self) -> bool { self.0.filter() }
}

impl UserData for ChordArc {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));

		fields.add_cached_field("on", |lua, me| lua.create_sequence_from(me.on.iter().copied()));

		fields.add_cached_field("run", |_, me| Ok(me.run.clone()));

		fields.add_cached_field("desc", |lua, me| lua.create_string(&me.desc));
	}
}
