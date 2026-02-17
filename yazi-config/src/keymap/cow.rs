use std::ops::Deref;

use yazi_shared::event::ActionCow;

use super::Chord;

#[derive(Clone, Debug)]
pub enum ChordCow {
	Owned(Chord),
	Borrowed(&'static Chord),
}

impl From<Chord> for ChordCow {
	fn from(c: Chord) -> Self { Self::Owned(c) }
}

impl From<&'static Chord> for ChordCow {
	fn from(c: &'static Chord) -> Self { Self::Borrowed(c) }
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
	fn default() -> Self {
		const C: &Chord = &Chord { on: vec![], run: vec![], desc: None, r#for: None };
		Self::Borrowed(C)
	}
}

impl ChordCow {
	pub fn into_seq(self) -> Vec<ActionCow> {
		match self {
			Self::Owned(c) => c.run.into_iter().rev().map(Into::into).collect(),
			Self::Borrowed(c) => c.run.iter().rev().map(Into::into).collect(),
		}
	}
}
