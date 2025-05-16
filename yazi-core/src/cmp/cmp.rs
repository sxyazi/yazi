use std::{collections::HashMap, path::PathBuf};

use yazi_proxy::options::CmpItem;
use yazi_shared::Id;

#[derive(Default)]
pub struct Cmp {
	pub(super) caches: HashMap<PathBuf, Vec<CmpItem>>,
	pub(super) cands:  Vec<CmpItem>,
	pub(super) offset: usize,
	pub cursor:        usize,

	pub(super) ticket: Id,
	pub visible:       bool,
}

impl Cmp {
	// --- Cands
	#[inline]
	pub fn window(&self) -> &[CmpItem] {
		let end = (self.offset + self.limit()).min(self.cands.len());
		&self.cands[self.offset..end]
	}

	#[inline]
	pub fn limit(&self) -> usize { self.cands.len().min(10) }

	#[inline]
	pub fn selected(&self) -> Option<&CmpItem> { self.cands.get(self.cursor) }

	// --- Cursor
	#[inline]
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }
}
