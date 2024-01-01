use std::ops::RangeBounds;

use yazi_shared::{event::Exec, render, CharKind};

use crate::input::Input;

pub struct Opt<'a> {
	kind: &'a str,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self { kind: e.args.first().map(|s| s.as_str()).unwrap_or_default() }
	}
}

impl Input {
	fn kill_range(&mut self, range: impl RangeBounds<usize>) {
		let snap = self.snap_mut();
		snap.cursor = match range.start_bound() {
			std::ops::Bound::Included(i) => *i,
			std::ops::Bound::Excluded(_) => unreachable!(),
			std::ops::Bound::Unbounded => 0,
		};
		if snap.value.drain(range).next().is_none() {
			return;
		}

		self.move_(0);
		self.flush_value();
		render!();
	}

	/// Searches for a word boundary and returns the movement in the cursor
	/// position.
	///
	/// A word boundary is where the [`CharKind`] changes.
	///
	/// If `skip_whitespace_first` is true, we skip initial whitespace.
	/// Otherwise, we skip whitespace after reaching a word boundary.
	///
	/// If `stop_before_boundary` is true, returns how many characters the cursor
	/// needs to move to be at the character *BEFORE* the word boundary, or until
	/// the end of the iterator.
	///
	/// Otherwise, returns how many characters to move to reach right *AFTER* the
	/// word boundary, or the end of the iterator.
	fn find_word_boundary(input: impl Iterator<Item = char> + Clone) -> usize {
		fn count_spaces(input: impl Iterator<Item = char>) -> usize {
			// Move until we don't see any more whitespace.
			input.take_while(|&c| CharKind::new(c) == CharKind::Space).count()
		}

		fn count_characters(mut input: impl Iterator<Item = char>) -> usize {
			// Determine the current character class.
			let first = match input.next() {
				Some(c) => CharKind::new(c),
				None => return 0,
			};

			// Move until we see a different character class or the end of the iterator.
			input.take_while(|&c| CharKind::new(c) == first).count() + 1
		}

		let spaces = count_spaces(input.clone());
		spaces + count_characters(input.skip(spaces))
	}

	pub fn kill<'a>(&mut self, opt: impl Into<Opt<'a>>) {
		let opt = opt.into() as Opt;
		let snap = self.snap_mut();

		match opt.kind.as_bytes() {
			b"bol" => {
				let end = snap.idx(snap.cursor).unwrap_or(snap.len());
				self.kill_range(..end)
			}
			b"eol" => {
				let start = snap.idx(snap.cursor).unwrap_or(snap.len());
				self.kill_range(start..)
			}
			b"backward" => {
				let end = snap.idx(snap.cursor).unwrap_or(snap.len());
				let start = end - Self::find_word_boundary(snap.value[..end].chars().rev());
				self.kill_range(start..end)
			}
			b"forward" => {
				let start = snap.idx(snap.cursor).unwrap_or(snap.len());
				let end = start + Self::find_word_boundary(snap.value[start..].chars());
				self.kill_range(start..end)
			}
			_ => {}
		}
	}
}
