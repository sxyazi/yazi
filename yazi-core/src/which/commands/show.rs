use yazi_config::{KEYMAP, keymap::Key};
use yazi_macro::render;
use yazi_parser::which::ShowOpt;
use yazi_shared::Layer;

use crate::which::{Which, WhichSorter};

impl Which {
	pub fn show(&mut self, opt: impl TryInto<ShowOpt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		if opt.cands.is_empty() {
			return;
		}

		self.times = 0;
		self.cands = opt.cands.into_iter().map(|c| c.into()).collect();

		self.visible = true;
		self.silent = opt.silent;
		render!();
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
