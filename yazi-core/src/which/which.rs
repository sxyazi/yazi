use std::mem;

use yazi_config::{keymap::{Control, Key}, KEYMAP};
use yazi_shared::{emit, render, Layer};

pub struct Which {
	layer:     Layer,
	pub times: usize,
	pub cands: Vec<&'static Control>,

	pub visible: bool,
}

impl Default for Which {
	fn default() -> Self {
		Self { layer: Layer::Manager, times: 0, cands: Default::default(), visible: false }
	}
}

impl Which {
	pub fn show(&mut self, key: &Key, layer: Layer) {
		self.layer = layer;
		self.times = 1;
		self.cands = KEYMAP.get(layer).iter().filter(|s| s.on.len() > 1 && &s.on[0] == key).collect();
		self.visible = true;
		render!();
	}

	pub fn type_(&mut self, key: Key) -> bool {
		self.cands = mem::take(&mut self.cands)
			.into_iter()
			.filter(|s| s.on.len() > self.times && s.on[self.times] == key)
			.collect();

		if self.cands.is_empty() {
			self.visible = false;
		} else if self.cands.len() == 1 {
			self.visible = false;
			emit!(Call(self.cands[0].to_call(), self.layer));
		} else if let Some(i) = self.cands.iter().position(|c| c.on.len() == self.times + 1) {
			self.visible = false;
			emit!(Call(self.cands[i].to_call(), self.layer));
		}

		self.times += 1;
		render!();

		true
	}
}
