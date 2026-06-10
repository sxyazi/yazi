use std::{ops::{Deref, DerefMut}, sync::Arc};

use serde::Deserialize;
use yazi_shared::{Layer, event::ActionCow};

use crate::{Mixable, keymap::Chord};

#[derive(Clone, Debug, Default, Deserialize)]
#[repr(transparent)]
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
