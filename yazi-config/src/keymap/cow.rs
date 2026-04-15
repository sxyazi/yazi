use std::ops::Deref;

use yazi_shared::{Layer, event::ActionCow};

use super::Chord;

#[derive(Clone, Debug)]
pub enum ChordCow<const L: u8 = { Layer::App as u8 }> {
	Owned(Chord<L>),
	Borrowed(&'static Chord<L>),
}

impl<const L: u8> From<Chord<L>> for ChordCow<L> {
	fn from(c: Chord<L>) -> Self { Self::Owned(c) }
}

impl<const L: u8> From<&'static Chord<L>> for ChordCow<L> {
	fn from(c: &'static Chord<L>) -> Self { Self::Borrowed(c) }
}

impl<const L: u8> Deref for ChordCow<L> {
	type Target = Chord<L>;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Owned(c) => c,
			Self::Borrowed(c) => c,
		}
	}
}

impl<const L: u8> Default for ChordCow<L> {
	fn default() -> Self { Self::Owned(Chord::default()) }
}

impl<const L: u8> ChordCow<L> {
	pub fn into_seq(self) -> Vec<ActionCow> {
		match self {
			Self::Owned(c) => c.run.into_iter().rev().map(Into::into).collect(),
			Self::Borrowed(c) => c.run.iter().rev().map(Into::into).collect(),
		}
	}
}
