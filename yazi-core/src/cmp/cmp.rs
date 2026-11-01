use std::io;

use hashbrown::HashMap;
use tokio::task::JoinHandle;
use yazi_shared::{id::Id, strand::AsStrand};
use yazi_widgets::{Scrollable, cmp::{CmpItem, CmpKey}};

use super::{CmpOpt, Matcher};

#[derive(Default)]
pub struct Cmp {
	pub caches:  HashMap<CmpKey, Vec<CmpItem>>,
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

	pub fn refresh(&mut self, opt: CmpOpt) -> bool {
		let Some(cache) = opt.cache.as_ref().or_else(|| self.caches.get(&opt.cache_key)) else {
			return false;
		};

		self.matches = Matcher::new(cache).matches(opt.word.as_strand()).cloned().collect();
		if let Some(cache) = opt.cache {
			self.caches.insert(opt.cache_key, cache);
		}

		!self.matches.is_empty()
	}

	// --- Cursor
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }
}

impl Scrollable for Cmp {
	fn total(&self) -> usize { self.matches.len() }

	fn limit(&self) -> usize { self.matches.len().min(10) }

	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
