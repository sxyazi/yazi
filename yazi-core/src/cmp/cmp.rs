use std::{collections::HashMap, path::PathBuf};

use yazi_proxy::options::CmpItem;
use yazi_shared::Id;
use yazi_widgets::Scrollable;

#[derive(Default)]
pub struct Cmp {
	pub caches: HashMap<PathBuf, Vec<CmpItem>>,
	pub cands:  Vec<CmpItem>,
	pub offset: usize,
	pub cursor: usize,

	pub ticket:  Id,
	pub visible: bool,
}

impl Cmp {
	// --- Cands
	#[inline]
	pub fn window(&self) -> &[CmpItem] {
		let end = (self.offset + self.limit()).min(self.cands.len());
		&self.cands[self.offset..end]
	}

	#[inline]
	pub fn selected(&self) -> Option<&CmpItem> { self.cands.get(self.cursor) }

	// --- Cursor
	#[inline]
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }
}

impl Scrollable for Cmp {
	#[inline]
	fn total(&self) -> usize { self.cands.len() }

	#[inline]
	fn limit(&self) -> usize { self.cands.len().min(10) }

	#[inline]
	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	#[inline]
	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
