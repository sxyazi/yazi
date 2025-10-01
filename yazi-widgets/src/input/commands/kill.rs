use std::ops::RangeBounds;

use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::input::KillOpt;
use yazi_shared::{CharKind, data::Data};

use crate::input::Input;

impl Input {
	pub fn kill(&mut self, opt: KillOpt) -> Result<Data> {
		let snap = self.snap_mut();
		match opt.kind.as_ref() {
			"all" => self.kill_range(..),
			"bol" => {
				let end = snap.idx(snap.cursor).unwrap_or(snap.len());
				self.kill_range(..end)
			}
			"eol" => {
				let start = snap.idx(snap.cursor).unwrap_or(snap.len());
				self.kill_range(start..)
			}
			"backward" => {
				let end = snap.idx(snap.cursor).unwrap_or(snap.len());
				let start = end - Self::find_word_boundary(snap.value[..end].chars().rev());
				self.kill_range(start..end)
			}
			"forward" => {
				let start = snap.idx(snap.cursor).unwrap_or(snap.len());
				let end = start + Self::find_word_boundary(snap.value[start..].chars());
				self.kill_range(start..end)
			}
			_ => succ!(),
		}
	}

	fn kill_range(&mut self, range: impl RangeBounds<usize>) -> Result<Data> {
		let snap = self.snap_mut();
		snap.cursor = match range.start_bound() {
			std::ops::Bound::Included(i) => *i,
			std::ops::Bound::Excluded(_) => unreachable!(),
			std::ops::Bound::Unbounded => 0,
		};
		if snap.value.drain(range).next().is_none() {
			succ!();
		}

		act!(r#move, self)?;
		self.flush_value();
		succ!(render!());
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

		let n = count_spaces(input.clone());
		let n = n + count_characters(input.clone().skip(n));
		input.take(n).fold(0, |acc, c| acc + c.len_utf8())
	}
}
