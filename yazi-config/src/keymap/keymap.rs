use std::sync::Arc;

use anyhow::{Context, Result};
use serde::Deserialize;
use yazi_codegen::{DeserializeOver, DeserializeOver1};
use yazi_fs::{Xdg, ok_or_not_found};
use yazi_shared::Layer;

use super::KeymapSection;
use crate::keymap::ChordArc;

#[derive(Deserialize, DeserializeOver, DeserializeOver1)]
pub struct Keymap {
	pub mgr:     KeymapSection,
	pub tasks:   KeymapSection,
	pub spot:    KeymapSection,
	pub pick:    KeymapSection,
	pub input:   KeymapSection,
	pub confirm: KeymapSection,
	pub help:    KeymapSection,
	pub cmp:     KeymapSection,
}

impl Keymap {
	pub fn chords(&self, layer: Layer) -> Arc<Vec<ChordArc>> {
		match self.section(layer) {
			Some(s) => s.load_full(),
			None => Arc::new(Vec::new()),
		}
	}

	pub fn section(&self, layer: Layer) -> Option<&KeymapSection> {
		use Layer as L;

		Some(match layer {
			L::Null | L::App => None?,
			L::Mgr => &self.mgr,
			L::Tasks => &self.tasks,
			L::Spot => &self.spot,
			L::Pick => &self.pick,
			L::Input => &self.input,
			L::Confirm => &self.confirm,
			L::Help => &self.help,
			L::Cmp => &self.cmp,
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
