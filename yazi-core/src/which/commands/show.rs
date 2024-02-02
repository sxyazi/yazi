use std::str::FromStr;

use yazi_config::{keymap::{Control, Key}, KEYMAP};
use yazi_shared::{event::Cmd, render, Layer};

use crate::which::Which;

pub struct Opt {
	cands:  Vec<Control>,
	layer:  Layer,
	silent: bool,
}

impl TryFrom<Cmd> for Opt {
	type Error = anyhow::Error;

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self {
			cands:  c.take_data().unwrap_or_default(),
			layer:  Layer::from_str(&c.take_name("layer").unwrap_or_default())?,
			silent: c.named.contains_key("silent"),
		})
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

		self.layer = opt.layer;
		self.times = 0;
		self.cands = opt.cands.into_iter().map(|c| c.into()).collect();

		self.visible = true;
		self.silent = opt.silent;
		render!();
	}

	pub fn show_with(&mut self, key: &Key, layer: Layer) {
		self.layer = layer;
		self.times = 1;
		self.cands = KEYMAP
			.get(layer)
			.iter()
			.filter(|c| c.on.len() > 1 && &c.on[0] == key)
			.map(|c| c.into())
			.collect();

		self.visible = true;
		self.silent = false;
		render!();
	}
}
