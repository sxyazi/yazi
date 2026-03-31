use std::io;

use hashbrown::HashMap;
use tokio::task::JoinHandle;
use yazi_shared::{Id, url::UrlBuf};
use yazi_widgets::Scrollable;

use crate::cmp::CmpItem;

#[derive(Default)]
pub struct Cmp {
	pub caches:  HashMap<UrlBuf, Vec<CmpItem>>,
	pub matches: Vec<CmpItem>,
	pub offset:  usize,
	pub cursor:  usize,

	pub ticket:  Id,
	pub handle:  Option<JoinHandle<io::Result<()>>>,
	pub visible: bool,
}

impl Cmp {
	// --- Matches
	pub fn window(&self) -> &[CmpItem] {
		let end = (self.offset + self.limit()).min(self.matches.len());
		&self.matches[self.offset..end]
	}

	pub fn selected(&self) -> Option<&CmpItem> { self.matches.get(self.cursor) }

	// --- Cursor
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }
}

impl Scrollable for Cmp {
	fn total(&self) -> usize { self.matches.len() }

	fn limit(&self) -> usize { self.matches.len().min(10) }

	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
