use yazi_config::{keymap::{Control, ControlCow, Key}, KEYMAP};
use yazi_shared::{emit, render, Layer};

pub struct Which {
	layer:     Layer,
	pub times: usize,
	pub cands: Vec<ControlCow>,

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
		self.cands = KEYMAP
			.get(layer)
			.iter()
			.filter(|c| c.on.len() > 1 && &c.on[0] == key)
			.map(|c| c.into())
			.collect();

		self.visible = true;
		render!();
	}

	pub fn show_with(&mut self, cands: Vec<Control>, layer: Layer) {
		self.layer = layer;
		self.times = 0;
		self.cands = cands.into_iter().map(|c| c.into()).collect();

		self.visible = true;
		render!();
	}

	pub fn type_(&mut self, key: Key) -> bool {
		self.cands.retain(|c| c.on.len() > self.times && c.on[self.times] == key);

		if self.cands.is_empty() {
			self.visible = false;
		} else if self.cands.len() == 1 {
			self.visible = false;
			emit!(Seq(self.cands[0].to_seq(), self.layer));
		} else if let Some(i) = self.cands.iter().position(|c| c.on.len() == self.times + 1) {
			self.visible = false;
			emit!(Seq(self.cands[i].to_seq(), self.layer));
		}

		self.times += 1;
		render!();

		true
	}
}
