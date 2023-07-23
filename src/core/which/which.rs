use std::mem;

use crate::{config::{keymap::{Control, Key, KeymapLayer}, KEYMAP}, emit};

pub struct Which {
	layer:     KeymapLayer,
	pub times: usize,
	pub cands: Vec<Control>,

	pub visible: bool,
}

impl Default for Which {
	fn default() -> Self {
		Self { layer: KeymapLayer::Manager, times: 0, cands: Default::default(), visible: false }
	}
}

impl Which {
	pub fn show(&mut self, key: &Key, layer: KeymapLayer) -> bool {
		self.layer = layer;
		self.times = 1;
		self.cands = KEYMAP
			.get(layer)
			.into_iter()
			.filter(|s| !s.on.is_empty() && s.on[0] == *key)
			.cloned()
			.collect();
		self.visible = true;
		true
	}

	pub fn press(&mut self, key: Key) -> bool {
		self.cands = mem::replace(&mut self.cands, Vec::new())
			.into_iter()
			.filter(|s| s.on.len() > self.times && s.on[self.times] == key)
			.collect();

		if self.cands.is_empty() {
			self.visible = false;
		} else if self.cands.len() == 1 {
			self.visible = false;
			emit!(Ctrl(self.cands.remove(0), self.layer));
		}

		self.times += 1;
		return true;
	}
}
