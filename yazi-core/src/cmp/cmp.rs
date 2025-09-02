use hashbrown::HashMap;
use yazi_parser::cmp::CmpItem;
use yazi_shared::{Id, url::UrlBuf};
use yazi_widgets::Scrollable;

#[derive(Default)]
pub struct Cmp {
	pub caches: HashMap<UrlBuf, Vec<CmpItem>>,
	pub cands:  Vec<CmpItem>,
	pub offset: usize,
	pub cursor: usize,

	pub ticket:  Id,
	pub visible: bool,
}

impl Cmp {
	// --- Cands
	pub fn window(&self) -> &[CmpItem] {
		let end = (self.offset + self.limit()).min(self.cands.len());
		&self.cands[self.offset..end]
	}

	pub fn selected(&self) -> Option<&CmpItem> { self.cands.get(self.cursor) }

	// --- Cursor
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }
}

impl Scrollable for Cmp {
	fn total(&self) -> usize { self.cands.len() }

	fn limit(&self) -> usize { self.cands.len().min(10) }

	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
