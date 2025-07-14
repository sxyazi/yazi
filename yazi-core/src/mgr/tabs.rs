use std::ops::{Deref, DerefMut};

use yazi_dds::Pubsub;
use yazi_fs::File;
use yazi_macro::err;

use crate::tab::{Folder, Tab};

pub struct Tabs {
	pub cursor: usize,
	pub items:  Vec<Tab>,
}

impl Default for Tabs {
	fn default() -> Self { Self { cursor: 0, items: vec![Default::default()] } }
}

impl Tabs {
	pub fn set_idx(&mut self, idx: usize) {
		// Reset the preview of the last active tab
		if let Some(active) = self.items.get_mut(self.cursor) {
			active.preview.reset_image();
		}

		self.cursor = idx;
		err!(Pubsub::pub_after_tab(self.active().id));
	}
}

impl Tabs {
	#[inline]
	pub fn active(&self) -> &Tab { &self[self.cursor] }

	#[inline]
	pub(super) fn active_mut(&mut self) -> &mut Tab { &mut self.items[self.cursor] }

	#[inline]
	pub fn parent(&self) -> Option<&Folder> { self.active().parent.as_ref() }

	#[inline]
	pub fn current(&self) -> &Folder { &self.active().current }

	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.current().hovered() }
}

impl Deref for Tabs {
	type Target = Vec<Tab>;

	fn deref(&self) -> &Self::Target { &self.items }
}

impl DerefMut for Tabs {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.items }
}
