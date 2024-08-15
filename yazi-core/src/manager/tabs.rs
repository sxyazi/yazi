use std::ops::{Deref, DerefMut};

use yazi_boot::BOOT;
use yazi_dds::Pubsub;
use yazi_proxy::ManagerProxy;
use yazi_shared::fs::Url;

use crate::tab::Tab;

pub struct Tabs {
	pub cursor:       usize,
	pub(super) items: Vec<Tab>,
}

impl Tabs {
	pub fn make() -> Self {
		let mut tabs =
			Self { cursor: 0, items: (0..BOOT.cwds.len()).map(|_| Tab::default()).collect() };
		tabs.reorder();

		for (i, tab) in tabs.iter_mut().enumerate() {
			let file = &BOOT.files[i];
			if file.is_empty() {
				tab.cd(Url::from(&BOOT.cwds[i]));
			} else {
				tab.reveal(Url::from(BOOT.cwds[i].join(file)));
			}
		}
		tabs
	}

	#[inline]
	pub(super) fn absolute(&self, rel: isize) -> usize {
		if rel > 0 {
			(self.cursor + rel as usize).min(self.items.len() - 1)
		} else {
			self.cursor.saturating_sub(rel.unsigned_abs())
		}
	}

	#[inline]
	pub(super) fn reorder(&mut self) {
		self.items.iter_mut().enumerate().for_each(|(i, tab)| tab.idx = i);
	}

	pub(super) fn set_idx(&mut self, idx: usize) {
		if self.cursor == idx {
			return;
		}

		// Reset the preview of the last active tab
		if let Some(active) = self.items.get_mut(self.cursor) {
			active.preview.reset_image();
		}

		self.cursor = idx;
		ManagerProxy::refresh();
		ManagerProxy::peek(true);
		Pubsub::pub_from_tab(idx);
	}
}

impl Tabs {
	#[inline]
	pub fn active(&self) -> &Tab { &self.items[self.cursor] }

	#[inline]
	pub(super) fn active_mut(&mut self) -> &mut Tab { &mut self.items[self.cursor] }

	#[inline]
	pub fn active_or(&self, idx: Option<usize>) -> &Tab {
		idx.and_then(|i| self.items.get(i)).unwrap_or(&self.items[self.cursor])
	}

	#[inline]
	pub(super) fn active_or_mut(&mut self, idx: Option<usize>) -> &mut Tab {
		if let Some(i) = idx.filter(|&i| i < self.items.len()) {
			&mut self.items[i]
		} else {
			&mut self.items[self.cursor]
		}
	}
}

impl Deref for Tabs {
	type Target = Vec<Tab>;

	fn deref(&self) -> &Self::Target { &self.items }
}

impl DerefMut for Tabs {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.items }
}
