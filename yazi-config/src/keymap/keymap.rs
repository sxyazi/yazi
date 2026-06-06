use std::{ops::Deref, sync::Arc};

use anyhow::{Context, Result};
use serde::Deserialize;
use yazi_codegen::{DeserializeOver, DeserializeOver1};
use yazi_fs::{Xdg, ok_or_not_found};
use yazi_shared::Layer;

use super::{Chord, KeymapSection};

#[derive(Deserialize, DeserializeOver, DeserializeOver1)]
pub struct Keymap {
	pub mgr:     KeymapSection<{ Layer::Mgr as u8 }>,
	pub tasks:   KeymapSection<{ Layer::Tasks as u8 }>,
	pub spot:    KeymapSection<{ Layer::Spot as u8 }>,
	pub pick:    KeymapSection<{ Layer::Pick as u8 }>,
	pub input:   KeymapSection<{ Layer::Input as u8 }>,
	pub confirm: KeymapSection<{ Layer::Confirm as u8 }>,
	pub help:    KeymapSection<{ Layer::Help as u8 }>,
	pub cmp:     KeymapSection<{ Layer::Cmp as u8 }>,
}

impl Keymap {
	pub fn get(&self, layer: Layer) -> Arc<Vec<Arc<Chord>>> {
		match layer {
			Layer::Null | Layer::App => Arc::new(Vec::new()),
			Layer::Mgr => self.mgr.deref().as_erased(),
			Layer::Tasks => self.tasks.deref().as_erased(),
			Layer::Spot => self.spot.deref().as_erased(),
			Layer::Pick => self.pick.deref().as_erased(),
			Layer::Input => self.input.deref().as_erased(),
			Layer::Confirm => self.confirm.deref().as_erased(),
			Layer::Help => self.help.deref().as_erased(),
			Layer::Cmp => self.cmp.deref().as_erased(),
			Layer::Which => Arc::new(Vec::new()),
			Layer::Notify => Arc::new(Vec::new()),
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
