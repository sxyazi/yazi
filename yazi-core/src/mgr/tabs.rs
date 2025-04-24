use std::ops::{Deref, DerefMut};

use yazi_boot::BOOT;
use yazi_dds::Pubsub;
use yazi_fs::File;
use yazi_proxy::MgrProxy;
use yazi_shared::{Id, url::Url};

use crate::tab::{Folder, Tab};

pub struct Tabs {
	pub cursor:       usize,
	pub(super) items: Vec<Tab>,
}

impl Tabs {
	pub fn make() -> Self {
		let mut tabs =
			Self { cursor: 0, items: (0..BOOT.cwds.len()).map(|_| Tab::default()).collect() };

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

	pub(super) fn set_idx(&mut self, idx: usize) {
		// Reset the preview of the last active tab
		if let Some(active) = self.items.get_mut(self.cursor) {
			active.preview.reset_image();
		}

		self.cursor = idx;
		MgrProxy::refresh();
		MgrProxy::peek(true);
		Pubsub::pub_from_tab(self.active().id);
	}
}

impl Tabs {
	#[inline]
	pub fn active(&self) -> &Tab { &self.items[self.cursor] }

	#[inline]
	pub(super) fn active_mut(&mut self) -> &mut Tab { &mut self.items[self.cursor] }

	#[inline]
	pub fn current(&self) -> &Folder { &self.active().current }

	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.current().hovered() }

	#[inline]
	pub fn find_mut(&mut self, id: Id) -> Option<&mut Tab> { self.iter_mut().find(|t| t.id == id) }
}

impl Deref for Tabs {
	type Target = Vec<Tab>;

	fn deref(&self) -> &Self::Target { &self.items }
}

impl DerefMut for Tabs {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.items }
}
