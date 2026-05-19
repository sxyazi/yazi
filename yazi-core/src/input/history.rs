use std::{collections::VecDeque, mem};

use yazi_widgets::input::InputSnaps;

// TODO: make configurable?
const MAX_LENGTH: usize = 20;

#[derive(Default)]
pub struct InputHistory {
	entries: VecDeque<String>,
	entry_snaps: VecDeque<Option<InputSnaps>>,
	idx: Option<usize>,
	draft: Option<InputSnaps>,
}

impl InputHistory {
	pub const fn new() -> Self {
		Self {
			entries: VecDeque::new(),
			entry_snaps: VecDeque::new(),
			idx: None,
			draft: None,
		}
	}

	pub fn push(&mut self, value: String) {
		if value.is_empty() {
			return;
		}
		if self.entries.back().map(String::as_str) != Some(&value) {
			if self.entries.len() >= MAX_LENGTH {
				self.entries.pop_front();
				self.entry_snaps.pop_front();
			}
			self.entries.push_back(value);
			self.entry_snaps.push_back(None);
		}
		self.reset();
	}

	pub fn reset(&mut self) {
		self.idx = None;
		self.draft = None;
	}

	pub fn navigate(&mut self, step: i64, snaps: &mut InputSnaps, limit: usize) -> bool {
		if self.entries.is_empty() || step == 0 {
			return false;
		}

		let len = self.entries.len() as i64;
		let pos = self.idx.map_or(len, |i| i as i64);
		let new_pos = (pos + step).clamp(0, len);

		if new_pos == pos {
			return false;
		}

		let mode = snaps.current().mode;
		let cursor = snaps.current().cursor;

		// Save current snaps into draft or the slot we're leaving
		let old = mem::take(snaps);
		if let Some(old_idx) = self.idx {
			self.entry_snaps[old_idx] = Some(old);
		} else {
			self.draft = Some(old);
		}

		// Load target snaps
		*snaps = if new_pos == len {
			self.idx = None;
			self.draft.take().unwrap_or_default()
		} else {
			let new_idx = new_pos as usize;
			self.idx = Some(new_idx);
			if self.entry_snaps[new_idx].is_none() {
				let value = self.entries[new_idx].clone();
				// history does not trigger on obscured inputs
				self.entry_snaps[new_idx] = Some(InputSnaps::new(value, false, limit));
			}
			self.entry_snaps[new_idx].take().unwrap()
		};

		// Preserve mode and cursor position from before navigation
		snaps.update_current(mode, cursor, limit);

		true
	}
}
