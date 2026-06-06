use std::{ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use serde::Deserialize;
use yazi_shared::Layer;
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, vec::{IndexAtError, VecExt}};

use super::Chord;
use crate::keymap::ChordMatcher;

#[derive(Debug, Default, Deserialize)]
pub struct Chords<const L: u8 = { Layer::Null as u8 }>(ArcSwap<Vec<Arc<Chord<L>>>>);

impl<const L: u8> Deref for Chords<L> {
	type Target = ArcSwap<Vec<Arc<Chord<L>>>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<const L: u8> From<Vec<Arc<Chord<L>>>> for Chords<L> {
	fn from(inner: Vec<Arc<Chord<L>>>) -> Self { Self(inner.into_pointee()) }
}

impl<const L: u8> Chords<L> {
	pub fn as_erased<const M: u8>(&self) -> Arc<Vec<Arc<Chord<M>>>> {
		let chords = self.0.load_full();
		unsafe { Arc::from_raw(Arc::into_raw(chords) as *const Vec<Arc<Chord<M>>>) }
	}

	pub fn insert(&self, index: isize, rule: Arc<Chord>) -> Result<(), IndexAtError> {
		self.0.try_rcu(|rules| {
			let (before, after) = rules.split_at(rules.index_at(index)?);
			Ok(
				before
					.iter()
					.cloned()
					.chain([rule.as_erased().clone()])
					.chain(after.iter().cloned())
					.collect::<Vec<_>>(),
			)
		})?;

		Ok(())
	}

	pub fn remove(&self, matcher: ChordMatcher) {
		self.0.rcu(|chords| {
			let mut next = Vec::clone(chords);
			next.retain(|chord| !matcher.matches(chord.as_erased()));
			next
		});
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<Arc<Chord<L>>> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique chords arc")
	}
}
