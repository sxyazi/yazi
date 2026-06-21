use std::{ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use mlua::{ExternalError, ExternalResult, MetaMethod, Table, UserData, UserDataMethods, Value};
use serde::Deserialize;
use yazi_shared::{Layer, event::Actions};
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, mlua::DeserializeOverLua, vec::{IndexAtError, VecExt}};

use crate::keymap::{Chord, ChordArc, ChordIter, ChordMatcher};

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Chords<const L: u8 = { Layer::Null as u8 }> {
	pub chords: ArcSwap<Vec<ChordArc<L>>>,
	#[serde(skip, default = "layer_default::<L>")]
	pub layer:  Layer,
}

impl<const L: u8> Default for Chords<L> {
	fn default() -> Self {
		Self { chords: ArcSwap::default(), layer: Layer::from_repr(L).unwrap_or_default() }
	}
}

impl<const L: u8> Deref for Chords<L> {
	type Target = ArcSwap<Vec<ChordArc<L>>>;

	fn deref(&self) -> &Self::Target { &self.chords }
}

impl<const L: u8> From<Vec<ChordArc<L>>> for Chords<L> {
	fn from(inner: Vec<ChordArc<L>>) -> Self {
		Self { chords: inner.into_pointee(), layer: Layer::from_repr(L).unwrap_or_default() }
	}
}

impl<const L: u8> Chords<L> {
	pub fn as_erased<const M: u8>(&self) -> Arc<Vec<ChordArc<M>>> {
		let chords = self.chords.load_full();
		unsafe { Arc::from_raw(Arc::into_raw(chords) as *const Vec<ChordArc<M>>) }
	}

	pub fn insert(&self, index: isize, rule: ChordArc) -> Result<(), IndexAtError> {
		self.chords.try_rcu(|rules| {
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
		self.chords.rcu(|chords| {
			let mut next = Vec::clone(chords);
			next.retain(|arc| !matcher.matches(arc.as_erased()));
			next
		});
	}

	pub fn update<E>(
		&self,
		matcher: ChordMatcher,
		f: impl Fn(Chord) -> Result<Chord, E>,
	) -> Result<(), E> {
		self.chords.try_rcu(|rules| {
			let mut next = Vec::clone(rules);
			for arc in &mut next {
				if matcher.matches(arc.as_erased()) {
					*arc = f(Chord::clone(arc.as_erased()))?.into();
				}
			}
			Ok(Arc::new(next))
		})?;

		Ok(())
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<ChordArc<L>> {
		Arc::try_unwrap(self.chords.into_inner()).expect("unique chords arc")
	}
}

impl UserData for &Chords {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |_, &me, matcher: Option<ChordMatcher>| {
			Ok(match matcher {
				Some(matcher) => ChordIter { chords: me.as_erased(), matcher, ..Default::default() },
				None => me.into(),
			})
		});

		methods.add_method("insert", |_, me, (index, value): (isize, Value)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			let chord = ChordArc::try_from((value, me.layer))?;
			me.insert(index, chord.clone()).into_lua_err()?;

			Ok(chord)
		});

		methods.add_method("remove", |_, me, matcher: ChordMatcher| {
			me.remove(matcher);
			Ok(())
		});

		methods.add_method("update", |_, me, (matcher, table): (ChordMatcher, Table)| {
			let mut run: Option<Actions> = table.raw_get("run")?;
			if let Some(run) = &mut run {
				table.raw_remove("run")?;
				run.set(me.layer, yazi_shared::Source::Key);
			}

			me.update(matcher, |mut chord| {
				chord = chord.deserialize_over_lua(&table)?;
				if let Some(run) = &run {
					chord.run = run.clone();
				}
				Ok(chord)
			})
		});

		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.load().len()));
	}
}

pub(super) fn layer_default<const L: u8>() -> Layer { Layer::from_repr(L).unwrap_or_default() }
