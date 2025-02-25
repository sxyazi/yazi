use yazi_config::{KEYMAP, keymap::{Chord, Key}};
use yazi_macro::render;
use yazi_shared::{Layer, event::CmdCow};

use crate::which::{Which, WhichSorter};

pub struct Opt {
	cands:  Vec<Chord>,
	silent: bool,
}

impl TryFrom<CmdCow> for Opt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { cands: c.take_any("candidates").unwrap_or_default(), silent: c.bool("silent") })
	}
}

impl Which {
	pub fn show(&mut self, opt: impl TryInto<Opt>) {
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
