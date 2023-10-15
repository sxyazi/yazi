use config::BOOT;
use shared::Url;

use crate::{emit, tab::Tab};

const MAX_TABS: usize = 9;

pub struct Tabs {
	pub idx: usize,
	items:   Vec<Tab>,
}

impl Tabs {
	pub fn make() -> Self {
		let mut tabs = Self { idx: usize::MAX, items: vec![Tab::from(Url::from(&BOOT.cwd))] };
		tabs.set_idx(0);
		tabs
	}

	pub fn create(&mut self, url: &Url) -> bool {
		if self.items.len() >= MAX_TABS {
			return false;
		}

		let mut tab = Tab::from(url);
		tab.set_show_hidden(Some(self.active().show_hidden));
		tab.set_sorter(self.active().sorter);

		self.items.insert(self.idx + 1, tab);
		self.set_idx(self.idx + 1);
		true
	}

	pub fn switch(&mut self, idx: isize, rel: bool) -> bool {
		let idx = if rel {
			(self.idx as isize + idx).rem_euclid(self.items.len() as isize) as usize
		} else {
			idx as usize
		};

		if idx == self.idx || idx >= self.items.len() {
			return false;
		}

		self.set_idx(idx);
		true
	}

	pub fn swap(&mut self, rel: isize) -> bool {
		let idx = self.absolute(rel);
		if idx == self.idx {
			return false;
		}

		self.items.swap(self.idx, idx);
		self.set_idx(idx);
		true
	}

	pub fn close(&mut self, idx: usize) -> bool {
		let len = self.items.len();
		if len < 2 || idx >= len {
			return false;
		}

		self.items.remove(idx);
		if idx <= self.idx {
			self.set_idx(self.absolute(1));
		}

		true
	}

	#[inline]
	fn absolute(&self, rel: isize) -> usize {
		if rel > 0 {
			(self.idx + rel as usize).min(self.items.len() - 1)
		} else {
			self.idx.saturating_sub(rel.unsigned_abs())
		}
	}

	#[inline]
	fn set_idx(&mut self, idx: usize) {
		self.idx = idx;
		self.active_mut().preview_reset_image();
		emit!(Refresh);
	}
}

impl Tabs {
	#[inline]
	pub fn len(&self) -> usize { self.items.len() }

	#[inline]
	pub fn iter(&self) -> impl Iterator<Item = &Tab> { self.items.iter() }

	#[inline]
	pub fn active(&self) -> &Tab { &self.items[self.idx] }

	#[inline]
	pub(super) fn active_mut(&mut self) -> &mut Tab { &mut self.items[self.idx] }
}
