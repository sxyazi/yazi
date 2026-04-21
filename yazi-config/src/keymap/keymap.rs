use anyhow::{Context, Result};
use serde::Deserialize;
use yazi_codegen::{DeserializeOver, DeserializeOver1};
use yazi_fs::{Xdg, ok_or_not_found};
use yazi_shared::Layer;

use super::{Chord, KeymapRules};

#[derive(Deserialize, DeserializeOver, DeserializeOver1)]
pub struct Keymap {
	pub mgr:     KeymapRules<{ Layer::Mgr as u8 }>,
	pub tasks:   KeymapRules<{ Layer::Tasks as u8 }>,
	pub spot:    KeymapRules<{ Layer::Spot as u8 }>,
	pub pick:    KeymapRules<{ Layer::Pick as u8 }>,
	pub input:   KeymapRules<{ Layer::Input as u8 }>,
	pub confirm: KeymapRules<{ Layer::Confirm as u8 }>,
	pub help:    KeymapRules<{ Layer::Help as u8 }>,
	pub cmp:     KeymapRules<{ Layer::Cmp as u8 }>,
}

impl Keymap {
	pub fn get(&self, layer: Layer) -> &[Chord] {
		match layer {
			Layer::Null | Layer::App => &[],
			Layer::Mgr => self.mgr.as_erased_slice(),
			Layer::Tasks => self.tasks.as_erased_slice(),
			Layer::Spot => self.spot.as_erased_slice(),
			Layer::Pick => self.pick.as_erased_slice(),
			Layer::Input => self.input.as_erased_slice(),
			Layer::Confirm => self.confirm.as_erased_slice(),
			Layer::Help => self.help.as_erased_slice(),
			Layer::Cmp => self.cmp.as_erased_slice(),
			Layer::Which => &[],
			Layer::Notify => &[],
		}
	}
}

impl Keymap {
	pub(crate) fn read() -> Result<String> {
		let p = Xdg::config_dir().join("keymap.toml");
		ok_or_not_found(std::fs::read_to_string(&p))
			.with_context(|| format!("Failed to read keymap {p:?}"))
	}
}
