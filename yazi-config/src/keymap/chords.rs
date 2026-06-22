use std::{ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use mlua::{ExternalError, ExternalResult, MetaMethod, Table, UserData, UserDataMethods};
use serde::Deserialize;
use yazi_shared::event::Actions;
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, mlua::DeserializeOverLua, vec::{IndexAtError, VecExt}};

use crate::keymap::{Chord, ChordArc, ChordIter, ChordMatcher};

#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Chords {
	pub(super) chords: ArcSwap<Vec<ChordArc>>,
}

impl Deref for Chords {
	type Target = ArcSwap<Vec<ChordArc>>;

	fn deref(&self) -> &Self::Target { &self.chords }
}

impl From<Vec<ChordArc>> for Chords {
	fn from(inner: Vec<ChordArc>) -> Self { Self { chords: inner.into_pointee() } }
}

impl Chords {
	pub fn insert(&self, index: isize, rule: ChordArc) -> Result<(), IndexAtError> {
		self.chords.try_rcu(|rules| {
			let (before, after) = rules.split_at(rules.index_at(index)?);
			Ok(
				before
					.iter()
					.cloned()
					.chain([rule.clone()])
					.chain(after.iter().cloned())
					.collect::<Vec<_>>(),
			)
		})?;

		Ok(())
	}

	pub fn remove(&self, matcher: ChordMatcher) {
		self.chords.rcu(|chords| {
			let mut next = Vec::clone(chords);
			next.retain(|arc| !matcher.matches(arc));
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
				if matcher.matches(arc) {
					*arc = f(Chord::clone(arc))?.into();
				}
			}
			Ok(Arc::new(next))
		})?;

		Ok(())
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<ChordArc> {
		Arc::try_unwrap(self.chords.into_inner()).expect("unique chords arc")
	}
}

impl UserData for &Chords {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |_, &me, matcher: Option<ChordMatcher>| {
			Ok(match matcher {
				Some(matcher) => ChordIter { chords: me.load_full(), matcher, ..Default::default() },
				None => me.into(),
			})
		});

		methods.add_method("insert", |_, me, (index, chord): (isize, ChordArc)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

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
				run.set_source(yazi_shared::Source::Key);
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
