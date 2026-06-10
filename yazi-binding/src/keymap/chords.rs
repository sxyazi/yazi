use std::ops::Deref;

use mlua::{ExternalError, ExternalResult, MetaMethod, Table, UserData, UserDataMethods, Value};
use yazi_shim::mlua::DeserializeOverLua;

use super::{Chord, ChordMatcher};
use crate::{event::Actions, keymap::ChordIter};

pub struct Chords {
	pub(super) inner: &'static yazi_config::keymap::Chords,
	pub(super) layer: yazi_shared::Layer,
}

impl Deref for Chords {
	type Target = yazi_config::keymap::Chords;

	fn deref(&self) -> &Self::Target { self.inner }
}

impl UserData for Chords {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |_, me, matcher: Option<ChordMatcher>| {
			Ok(match matcher {
				Some(matcher) => ChordIter::new(yazi_config::keymap::ChordIter {
					chords: me.as_erased(),
					matcher: matcher.0,
					..Default::default()
				}),
				None => ChordIter::new(&*me.inner),
			})
		});

		methods.add_method("insert", |_, me, (index, value): (isize, Value)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			let chord = Chord::try_from((value, me.layer))?;
			me.insert(index, chord.clone()).into_lua_err()?;

			Ok(chord)
		});

		methods.add_method("remove", |_, me, matcher: ChordMatcher| {
			me.remove(matcher.0);
			Ok(())
		});

		methods.add_method("update", |_, me, (matcher, table): (ChordMatcher, Table)| {
			let mut run: Option<Actions> = table.raw_get("run")?;
			if let Some(run) = &mut run {
				table.raw_remove("run")?;
				run.set(me.layer, yazi_shared::Source::Key);
			}

			me.update(matcher.0, |mut chord| {
				chord = chord.deserialize_over_lua(&table)?;
				if let Some(run) = &run {
					chord.run = run.clone().into();
				}
				Ok(chord)
			})
		});

		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.load().len()));
	}
}
