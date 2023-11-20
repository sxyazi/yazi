use yazi_config::BOOT;
use yazi_shared::Url;

use crate::{manager::Manager, tab::Tab};

pub struct Tabs {
	pub idx:          usize,
	pub(super) items: Vec<Tab>,
}

impl Tabs {
	pub fn make() -> Self {
		let mut tabs = Self { idx: usize::MAX, items: vec![Tab::from(Url::from(&BOOT.cwd))] };
		if let Some(file) = &BOOT.file {
			tabs.items[0].reveal(Url::from(BOOT.cwd.join(file)));
		}

		tabs.set_idx(0);
		tabs
	}

	#[inline]
	pub(super) fn absolute(&self, rel: isize) -> usize {
		if rel > 0 {
			(self.idx + rel as usize).min(self.items.len() - 1)
		} else {
			self.idx.saturating_sub(rel.unsigned_abs())
		}
	}

	#[inline]
	pub(super) fn set_idx(&mut self, idx: usize) {
		self.idx = idx;
		self.active_mut().preview.reset(|l| l.is_image());
		Manager::_refresh();
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
