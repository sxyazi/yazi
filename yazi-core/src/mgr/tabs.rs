use std::ops::{Deref, DerefMut};

use yazi_dds::Pubsub;
use yazi_macro::log_if_err;
use yazi_shared::id::Id;

use crate::tab::Tab;

pub struct Tabs {
	pub cursor: usize,
	pub items:  Vec<Tab>,
}

impl Default for Tabs {
	fn default() -> Self { Self { cursor: 0, items: vec![Default::default()] } }
}

impl Deref for Tabs {
	type Target = Vec<Tab>;

	fn deref(&self) -> &Self::Target { &self.items }
}

impl DerefMut for Tabs {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.items }
}

impl Tabs {
	#[inline]
	pub fn idx(&self, id: Id) -> Option<usize> { self.items.iter().position(|tab| tab.id == id) }

	pub fn set_idx(&mut self, idx: usize) {
		// Reset the preview of the last active tab
		if let Some(active) = self.items.get_mut(self.cursor) {
			active.preview.reset_image();
		}

		self.cursor = idx;
		log_if_err!(Pubsub::pub_after_tab(self.active().id));
	}

	pub fn indices_or_active(&self, ids: Vec<Id>) -> Vec<usize> {
		if ids.is_empty() {
			vec![self.cursor]
		} else {
			ids.into_iter().filter_map(|id| self.idx(id)).collect()
		}
	}
}

impl Tabs {
	#[inline]
	pub(crate) fn active(&self) -> &Tab { &self[self.cursor] }

	#[inline]
	pub(super) fn active_mut(&mut self) -> &mut Tab { &mut self.items[self.cursor] }
}
