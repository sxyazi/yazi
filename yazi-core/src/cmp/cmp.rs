use std::{collections::HashMap, path::PathBuf};

#[derive(Default)]
pub struct Cmp {
	pub(super) caches: HashMap<PathBuf, Vec<String>>,
	pub(super) cands:  Vec<String>,
	pub(super) offset: usize,
	pub cursor:        usize,

	pub(super) ticket: usize,
	pub visible:       bool,
}

impl Cmp {
	// --- Cands
	#[inline]
	pub fn window(&self) -> &[String] {
		let end = (self.offset + self.limit()).min(self.cands.len());
		&self.cands[self.offset..end]
	}

	#[inline]
	pub fn limit(&self) -> usize { self.cands.len().min(10) }

	#[inline]
	pub fn selected(&self) -> Option<&String> { self.cands.get(self.cursor) }

	// --- Cursor
	#[inline]
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }
}
