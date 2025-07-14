use yazi_config::{KEYMAP, keymap::{ChordCow, Key}};
use yazi_macro::{emit, render, render_and};
use yazi_shared::Layer;

use crate::which::WhichSorter;

#[derive(Default)]
pub struct Which {
	pub times: usize,
	pub cands: Vec<ChordCow>,

	// Visibility
	pub visible: bool,
	pub silent:  bool,
}

impl Which {
	pub fn r#type(&mut self, key: Key) -> bool {
		self.cands.retain(|c| c.on.len() > self.times && c.on[self.times] == key);
		self.times += 1;

		if self.cands.is_empty() {
			self.reset();
		} else if self.cands.len() == 1 {
			emit!(Seq(self.cands.remove(0).into_seq()));
			self.reset();
		} else if let Some(i) = self.cands.iter().position(|c| c.on.len() == self.times) {
			emit!(Seq(self.cands.remove(i).into_seq()));
			self.reset();
		}

		render_and!(true)
	}

	fn reset(&mut self) {
		self.times = 0;
		self.cands.clear();

		self.visible = false;
		self.silent = false;
	}

	pub fn show_with(&mut self, key: Key, layer: Layer) {
		self.times = 1;
		self.cands = KEYMAP
			.get(layer)
			.iter()
			.filter(|c| c.on.len() > 1 && c.on[0] == key)
			.map(|c| c.into())
			.collect();

		WhichSorter::default().sort(&mut self.cands);
		self.visible = true;
		self.silent = false;
		render!();
	}
}
