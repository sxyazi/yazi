use yazi_config::keymap::{ControlCow, Key};
use yazi_shared::{emit, render, Layer};

#[derive(Default)]
pub struct Which {
	pub(super) layer: Layer,
	pub times:        usize,
	pub cands:        Vec<ControlCow>,

	pub visible: bool,
	pub silent:  bool,
}

impl Which {
	fn reset(&mut self) {
		self.times = 0;
		self.cands.clear();

		self.visible = false;
		self.silent = false;
	}

	pub fn type_(&mut self, key: Key) -> bool {
		self.cands.retain(|c| c.on.len() > self.times && c.on[self.times] == key);
		self.times += 1;

		if self.cands.is_empty() {
			self.reset();
		} else if self.cands.len() == 1 {
			emit!(Seq(self.cands.remove(0).into_seq(), self.layer));
			self.reset();
		} else if let Some(i) = self.cands.iter().position(|c| c.on.len() == self.times) {
			emit!(Seq(self.cands.remove(i).into_seq(), self.layer));
			self.reset();
		}

		render!();
		true
	}
}
