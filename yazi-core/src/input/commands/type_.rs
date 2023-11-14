use std::ops::RangeBounds;

use crossterm::event::KeyCode;
use yazi_config::keymap::{Exec, Key};
use yazi_shared::CharKind;

use crate::input::{Input, InputMode};

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Input {
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

	fn delete_range(&mut self, range: impl RangeBounds<usize>) -> bool {
		let snap = self.snap_mut();
		snap.cursor = match range.start_bound() {
			std::ops::Bound::Included(i) => *i,
			std::ops::Bound::Excluded(_) => unreachable!(),
			std::ops::Bound::Unbounded => 0,
		};
		if snap.value.drain(range).next().is_some() {
			self.flush_value();
			return true;
		}
		false
	}

	fn backspace(&mut self, under: bool) -> bool {
		let snap = self.snaps.current_mut();
		if !under && snap.cursor < 1 {
			return false;
		} else if under && snap.cursor >= snap.value.len() {
			return false;
		}

		if under {
			snap.value.remove(snap.idx(snap.cursor).unwrap());
			self.move_(0);
		} else {
			snap.value.remove(snap.idx(snap.cursor - 1).unwrap());
			self.move_(-1);
		}

		self.flush_value();
		true
	}

	pub fn type_(&mut self, key: &Key) -> bool {
		if self.mode() != InputMode::Insert {
			return false;
		}

		if let Some(c) = key.plain() {
			let mut bits = [0; 4];
			return self.type_str(c.encode_utf8(&mut bits));
		}

		use KeyCode::{Backspace, Char as C};

		match key {
			// Move to the start of the line
			Key { code: C('a'), shift: false, ctrl: true, alt: false } => self.move_(isize::MIN),
			// Move to the end of the line
			Key { code: C('e'), shift: false, ctrl: true, alt: false } => self.move_(isize::MAX),

			// Move back a character
			Key { code: C('b'), shift: false, ctrl: true, alt: false } => self.move_(-1),
			// Move forward a character
			Key { code: C('f'), shift: false, ctrl: true, alt: false } => self.move_(1),

			// Delete the character before the cursor
			Key { code: Backspace, shift: false, ctrl: false, alt: false } => self.backspace(false),
			Key { code: C('h'), shift: false, ctrl: true, alt: false } => self.backspace(false),
			// Delete the character under the cursor
			Key { code: C('d'), shift: false, ctrl: true, alt: false } => self.backspace(true),

			// Move back to the start of the current or previous word
			Key { code: C('b'), shift: false, ctrl: false, alt: true } => {
				let snap = self.snap();
				let idx = snap.idx(snap.cursor).unwrap_or(snap.len());

				let step = Self::find_word_boundary(snap.value[..idx].chars().rev());
				self.move_(-(step as isize))
			}
			// Move forward to the end of the next word
			Key { code: C('f'), shift: false, ctrl: false, alt: true } => {
				let snap = self.snap();
				let step = Self::find_word_boundary(snap.value.chars().skip(snap.cursor));
				self.move_(step as isize)
			}

			// Kill backwards to the start of the line
			Key { code: C('u'), shift: false, ctrl: true, alt: false } => {
				let snap = self.snap_mut();
				let end = snap.idx(snap.cursor).unwrap_or(snap.len());
				self.delete_range(..end)
			}
			// Kill forwards to the end of the line
			Key { code: C('k'), shift: false, ctrl: true, alt: false } => {
				let snap = self.snap_mut();
				let start = snap.idx(snap.cursor).unwrap_or(snap.len());
				self.delete_range(start..)
			}

			// Kill backwards to the start of the current word
			Key { code: C('w'), shift: false, ctrl: true, alt: false }
			| Key { code: Backspace, shift: false, ctrl: false, alt: true } => {
				let snap = self.snap_mut();
				let end = snap.idx(snap.cursor).unwrap_or(snap.len());
				let start = end - Self::find_word_boundary(snap.value[..end].chars().rev());
				self.delete_range(start..end)
			}
			// Kill forwards to the end of the current word
			Key { code: C('d'), shift: false, ctrl: false, alt: true } => {
				let snap = self.snap_mut();
				let start = snap.idx(snap.cursor).unwrap_or(snap.len());
				let end = start + Self::find_word_boundary(snap.value[start..].chars());
				self.delete_range(start..end)
			}

			_ => false,
		}
	}
}
