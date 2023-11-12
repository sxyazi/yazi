use std::ops::{Range, RangeBounds};

use crossterm::event::KeyCode;
use tokio::sync::mpsc::UnboundedSender;
use unicode_width::UnicodeWidthStr;
use yazi_config::keymap::Key;
use yazi_shared::{CharKind, InputError};

use super::{mode::InputMode, op::InputOp, InputOpt, InputSnap, InputSnaps};
use crate::{external, Position};

#[derive(Default)]
pub struct Input {
	pub(super) snaps: InputSnaps,
	pub ticket:       usize,
	pub visible:      bool,

	pub title:    String,
	pub position: Position,

	// Typing
	pub(super) callback:   Option<UnboundedSender<Result<String, InputError>>>,
	realtime:              bool,
	pub(super) completion: bool,

	// Shell
	pub(super) highlight: bool,
}

impl Input {
	pub fn show(&mut self, opt: InputOpt, tx: UnboundedSender<Result<String, InputError>>) {
		self.close(false);
		self.snaps.reset(opt.value);
		self.visible = true;

		self.title = opt.title;
		self.position = opt.position;

		// Typing
		self.callback = Some(tx);
		self.realtime = opt.realtime;
		self.completion = opt.completion;

		// Shell
		self.highlight = opt.highlight;
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
	pub(super) fn find_word_boundary(input: impl Iterator<Item = char> + Clone) -> usize {
		fn count_spaces(input: impl Iterator<Item = char>) -> usize {
			// Move until we don't see any more whitespace.
			input.take_while(|c| CharKind::new(*c) == CharKind::Space).count()
		}

		fn count_characters(mut input: std::iter::Peekable<impl Iterator<Item = char>>) -> usize {
			// Determine the current character class.
			let prev = input.peek().cloned();
			let Some(prev) = prev else {
				return 0;
			};

			// Move until we see a different character class or the end of the iterator.
			input.take_while(|c| CharKind::new(*c) == CharKind::new(prev)).count()
		}

		let spaces_count = count_spaces(input.clone());
		let character_count = count_characters(input.skip(spaces_count).peekable());
		spaces_count + character_count
	}

	fn delete_range(&mut self, range: impl RangeBounds<usize>) {
		let snap = self.snap_mut();
		snap.cursor = match range.start_bound() {
			std::ops::Bound::Included(i) => *i,
			std::ops::Bound::Excluded(i) => i + 1,
			std::ops::Bound::Unbounded => 0,
		};
		snap.value.drain(range);
		self.flush_value();
	}

	pub fn type_(&mut self, key: &Key) -> bool {
		if self.mode() != InputMode::Insert {
			return false;
		}

		if let Some(c) = key.plain() {
			let mut bits = [0; 4];
			return self.type_str(c.encode_utf8(&mut bits));
		}

		use KeyCode::{Backspace, Char as C, Delete};

		match key {
			Key { code: Backspace, shift: false, ctrl: false, alt: false } => self.backspace(),
			// Handle Emacs-style keybindings.
			Key { code: C('a'), shift: false, ctrl: true, alt: false } => self.move_(isize::MIN),
			Key { code: C('e'), shift: false, ctrl: true, alt: false } => self.move_(isize::MAX),
			Key { code: C('b'), shift: false, ctrl: true, alt: false } => self.move_(-1),
			Key { code: C('f'), shift: false, ctrl: true, alt: false } => self.move_(1),
			Key { code: C('h'), shift: false, ctrl: true, alt: false } => self.backspace(),
			Key { code: C('d'), shift: false, ctrl: true, alt: false } => self.forward_delete(),
			Key { code: C('b'), shift: false, ctrl: false, alt: true } => {
				let snap = self.snap();
				let idx = snap.idx(snap.cursor).unwrap_or(snap.len());

				let step = Self::find_word_boundary(snap.value[..idx].chars().rev());
				self.move_(-(step as isize))
			}
			Key { code: C('f'), shift: false, ctrl: false, alt: true } => {
				let snap = self.snap();
				let idx = snap.idx(snap.cursor).unwrap_or(snap.len());

				let step = Self::find_word_boundary(snap.value[idx..].chars());
				self.move_(step as isize)
			}
			Key { code: C('u'), shift: false, ctrl: true, alt: false } => {
				let snap = self.snap_mut();
				let end = snap.idx(snap.cursor).unwrap_or(snap.len());
				self.delete_range(..end);
				true
			}
			Key { code: C('k'), shift: false, ctrl: true, alt: false }
			| Key { code: Delete, shift: false, ctrl: false, alt: false } => {
				let snap = self.snap_mut();
				let start = snap.idx(snap.cursor).unwrap_or(snap.len());
				self.delete_range(start..);
				true
			}
			Key { code: C('w'), shift: false, ctrl: true, alt: false }
			| Key { code: Backspace, shift: false, ctrl: false, alt: true } => {
				let snap = self.snap_mut();
				let end = snap.idx(snap.cursor).unwrap_or(snap.len());
				let start = end - Self::find_word_boundary(snap.value[..end].chars().rev());
				self.delete_range(start..end);
				true
			}
			Key { code: C('d'), shift: false, ctrl: false, alt: true } => {
				let snap = self.snap_mut();
				let start = snap.idx(snap.cursor).unwrap_or(snap.len());
				// Hitting this keybind `ab |cd `should give `|cd`.
				let end = start + Self::find_word_boundary(snap.value[start..].chars());
				self.delete_range(start..end);
				true
			}
			_ => false,
		}
	}

	pub fn forward_delete(&mut self) -> bool {
		let snap = self.snaps.current_mut();
		// Return false when there is no character on the right to delete.
		// Note that the cursor can be at index `snap.value.len()` when in
		// edit mode, but it should be strictly less for forward deletion.
		if snap.cursor >= snap.value.len() {
			return false;
		} else {
			snap.value.remove(snap.idx(snap.cursor).unwrap());
		}
		self.move_(0);
		self.flush_value();
		true
	}

	pub fn type_str(&mut self, s: &str) -> bool {
		let snap = self.snaps.current_mut();
		if snap.cursor < 1 {
			snap.value.insert_str(0, s);
		} else {
			snap.value.insert_str(snap.idx(snap.cursor).unwrap(), s);
		}

		self.move_(s.chars().count() as isize);
		self.flush_value();
		true
	}

	pub fn backspace(&mut self) -> bool {
		let snap = self.snaps.current_mut();
		if snap.cursor < 1 {
			return false;
		} else {
			snap.value.remove(snap.idx(snap.cursor - 1).unwrap());
		}

		self.move_(-1);
		self.flush_value();
		true
	}

	pub(super) fn handle_op(&mut self, cursor: usize, include: bool) -> bool {
		let old = self.snap().clone();
		let snap = self.snaps.current_mut();

		match snap.op {
			InputOp::None | InputOp::Select(_) => {
				snap.cursor = cursor;
			}
			InputOp::Delete(cut, insert, _) => {
				let range = snap.op.range(cursor, include).unwrap();
				let Range { start, end } = snap.idx(range.start)..snap.idx(range.end);

				let drain = snap.value.drain(start.unwrap()..end.unwrap()).collect::<String>();
				if cut {
					futures::executor::block_on(external::clipboard_set(&drain)).ok();
				}

				snap.op = InputOp::None;
				snap.mode = if insert { InputMode::Insert } else { InputMode::Normal };
				snap.cursor = range.start;
			}
			InputOp::Yank(_) => {
				let range = snap.op.range(cursor, include).unwrap();
				let Range { start, end } = snap.idx(range.start)..snap.idx(range.end);
				let yanked = &snap.value[start.unwrap()..end.unwrap()];

				snap.op = InputOp::None;
				futures::executor::block_on(external::clipboard_set(yanked)).ok();
			}
		};

		snap.cursor = snap.count().saturating_sub(snap.mode.delta()).min(snap.cursor);
		if snap == &old {
			return false;
		}
		if !matches!(old.op, InputOp::None | InputOp::Select(_)) {
			self.snaps.tag().then(|| self.flush_value());
		}
		true
	}

	#[inline]
	pub(super) fn flush_value(&mut self) {
		self.ticket = self.ticket.wrapping_add(1);

		if self.realtime {
			let value = self.snap().value.clone();
			self.callback.as_ref().unwrap().send(Err(InputError::Typed(value))).ok();
		}

		if self.completion {
			let before = self.partition()[0].to_owned();
			self.callback.as_ref().unwrap().send(Err(InputError::Completed(before, self.ticket))).ok();
		}
	}
}

impl Input {
	#[inline]
	pub fn value(&self) -> &str { self.snap().slice(self.snap().window()) }

	#[inline]
	pub fn mode(&self) -> InputMode { self.snap().mode }

	#[inline]
	pub fn cursor(&self) -> u16 {
		let snap = self.snap();
		snap.slice(snap.offset..snap.cursor).width() as u16
	}

	pub fn selected(&self) -> Option<Range<u16>> {
		let snap = self.snap();
		let start = snap.op.start()?;

		let (start, end) =
			if start < snap.cursor { (start, snap.cursor) } else { (snap.cursor + 1, start + 1) };

		let win = snap.window();
		let Range { start, end } = start.max(win.start)..end.min(win.end);

		let s = snap.slice(snap.offset..start).width() as u16;
		Some(s..s + snap.slice(start..end).width() as u16)
	}

	#[inline]
	pub fn partition(&self) -> [&str; 2] {
		let snap = self.snap();
		let idx = snap.idx(snap.cursor).unwrap();
		[&snap.value[..idx], &snap.value[idx..]]
	}

	#[inline]
	pub(super) fn snap(&self) -> &InputSnap { self.snaps.current() }

	#[inline]
	pub(super) fn snap_mut(&mut self) -> &mut InputSnap { self.snaps.current_mut() }
}
