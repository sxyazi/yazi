use std::{collections::VecDeque, ops::Deref};

use yazi_shared::event::Cmd;

use super::Chord;

#[derive(Debug)]
pub enum ChordCow {
	Owned(Chord),
	Borrowed(&'static Chord),
}

impl From<&'static Chord> for ChordCow {
	fn from(c: &'static Chord) -> Self { Self::Borrowed(c) }
}

impl From<Chord> for ChordCow {
	fn from(c: Chord) -> Self { Self::Owned(c) }
}

impl Deref for ChordCow {
	type Target = Chord;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Owned(c) => c,
			Self::Borrowed(c) => c,
		}
	}
}

impl Default for ChordCow {
	fn default() -> Self { Self::Owned(Chord::default()) }
}

impl ChordCow {
	pub fn into_seq(self) -> VecDeque<Cmd> {
		match self {
			Self::Owned(c) => c.run.into(),
			Self::Borrowed(c) => c.to_seq(),
		}
	}
}
