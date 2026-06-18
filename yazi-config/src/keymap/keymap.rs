use std::{ops::Deref, sync::Arc};

use anyhow::{Context, Result};
use serde::Deserialize;
use yazi_codegen::{DeserializeOver, DeserializeOver1};
use yazi_fs::{Xdg, ok_or_not_found};
use yazi_shared::Layer;

use super::KeymapSection;
use crate::keymap::ChordArc;

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
	pub fn chords(&self, layer: Layer) -> Arc<Vec<ChordArc>> {
		match self.section(layer) {
			Some(s) => s.deref().as_erased(),
			None => Arc::new(Vec::new()),
		}
	}

	pub fn section(&self, layer: Layer) -> Option<&KeymapSection> {
		use Layer as L;

		Some(match layer {
			L::Null | L::App => None?,
			L::Mgr => self.mgr.as_erased(),
			L::Tasks => self.tasks.as_erased(),
			L::Spot => self.spot.as_erased(),
			L::Pick => self.pick.as_erased(),
			L::Input => self.input.as_erased(),
			L::Confirm => self.confirm.as_erased(),
			L::Help => self.help.as_erased(),
			L::Cmp => self.cmp.as_erased(),
			L::Which => None?,
			L::Notify => None?,
		})
	}
}

impl Keymap {
	pub(crate) fn read() -> Result<String> {
		let p = Xdg::config_dir().join("keymap.toml");
		ok_or_not_found(std::fs::read_to_string(&p))
			.with_context(|| format!("Failed to read keymap {p:?}"))
	}
}
