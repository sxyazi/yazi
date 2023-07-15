use std::path::Path;

use super::Tab;
use crate::{config::MANAGER, emit};

const MAX_TABS: usize = 9;

pub struct Tabs {
	idx:   usize,
	items: Vec<Tab>,
}

impl Tabs {
	pub fn new() -> Self {
		let tabs = Self { idx: 0, items: vec![Tab::new(&MANAGER.cwd)] };

		emit!(Refresh);
		tabs
	}

	pub fn create(&mut self, path: &Path) -> bool {
		if self.items.len() >= MAX_TABS {
			return false;
		}

		self.items.insert(self.idx + 1, Tab::new(path));
		self.set_idx(self.idx + 1);
		true
	}

	pub fn switch(&mut self, idx: isize, rel: bool) -> bool {
		let idx = if rel { self.absolute(idx) } else { idx as usize };

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
		if len < 2 || idx as usize >= len {
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
			self.idx.saturating_sub(rel.abs() as usize)
		}
	}

	#[inline]
	fn set_idx(&mut self, idx: usize) {
		self.idx = idx;
		emit!(Refresh);
	}
}

impl Tabs {
	#[inline]
	pub fn idx(&self) -> usize { self.idx }

	#[inline]
	pub fn len(&self) -> usize { self.items.len() }

	#[inline]
	pub fn iter(&self) -> impl Iterator<Item = &Tab> { self.items.iter() }

	#[inline]
	pub fn active(&self) -> &Tab { &self.items[self.idx] }

	#[inline]
	pub(super) fn active_mut(&mut self) -> &mut Tab { &mut self.items[self.idx] }
}
