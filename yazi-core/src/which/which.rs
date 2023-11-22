use std::mem;

use yazi_config::{keymap::{Control, Key, KeymapLayer}, KEYMAP};

use crate::emit;

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
		self.cands =
			KEYMAP.get(layer).iter().filter(|s| s.on.len() > 1 && &s.on[0] == key).cloned().collect();
		self.switch(true);
		true
	}

	pub fn press(&mut self, key: Key) -> bool {
		self.cands = mem::take(&mut self.cands)
			.into_iter()
			.filter(|s| s.on.len() > self.times && s.on[self.times] == key)
			.collect();

		if self.cands.is_empty() {
			self.switch(false);
		} else if self.cands.len() == 1 {
			self.switch(false);
			emit!(Call(self.cands[0].to_call(), self.layer));
		} else if let Some(i) = self.cands.iter().position(|c| c.on.len() == self.times + 1) {
			self.switch(false);
			emit!(Call(self.cands[i].to_call(), self.layer));
		}

		self.times += 1;
		true
	}

	#[inline]
	fn switch(&mut self, state: bool) {
		self.visible = state;
		// TODO: Peek
		// emit!(Peek); // Show/hide preview for images
	}
}
