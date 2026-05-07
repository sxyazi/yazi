use std::mem;

use super::{InputMode, InputSnaps};

#[derive(Default)]
pub struct InputHistory {
	entries: Vec<String>,
	entry_snaps: Vec<Option<InputSnaps>>,
	idx: Option<usize>,
	draft: Option<InputSnaps>,
}

impl InputHistory {
	pub const fn new() -> Self {
		Self { entries: Vec::new(), entry_snaps: Vec::new(), idx: None, draft: None }
	}

	pub fn push(&mut self, value: String) {
		if value.is_empty() {
			return;
		}
		if self.entries.last().map(String::as_str) != Some(&value) {
			self.entries.push(value);
			self.entry_snaps.push(None);
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
				self.entry_snaps[new_idx] = Some(Self::initial_snaps(value, mode, limit));
			}
			self.entry_snaps[new_idx].take().unwrap()
		};

		true
	}

	fn initial_snaps(value: String, mode: InputMode, limit: usize) -> InputSnaps {
		let mut snaps = InputSnaps::new(value, false, limit);
		let snap = snaps.current_mut();
		snap.mode = mode;
		snap.cursor = snap.count().saturating_sub(mode.delta());
		snap.resize(limit);
		snaps
	}
}
