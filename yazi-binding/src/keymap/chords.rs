use std::ops::Deref;

use mlua::{ExternalError, ExternalResult, MetaMethod, UserData, UserDataMethods};

use super::{Chord, ChordMatcher};
use crate::keymap::ChordIter;

pub struct Chords {
	inner: &'static yazi_config::keymap::Chords,
}

impl Deref for Chords {
	type Target = yazi_config::keymap::Chords;

	fn deref(&self) -> &Self::Target { self.inner }
}

impl Chords {
	pub fn new(inner: &'static yazi_config::keymap::Chords) -> Self { Self { inner } }
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

		methods.add_method("insert", |_, me, (index, chord): (isize, Chord)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			me.insert(index, chord.clone()).into_lua_err()?;
			Ok(chord)
		});

		methods.add_method("remove", |_, me, matcher: ChordMatcher| {
			me.remove(matcher.0);
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.load().len()));
	}
}
