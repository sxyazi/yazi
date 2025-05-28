use anyhow::Result;
use serde::Deserialize;
use yazi_codegen::DeserializeOver1;
use yazi_shared::Layer;

use super::{Chord, KeymapRules};

#[derive(Deserialize, DeserializeOver1)]
pub struct Keymap {
	pub mgr:     KeymapRules,
	pub tasks:   KeymapRules,
	pub spot:    KeymapRules,
	pub pick:    KeymapRules,
	pub input:   KeymapRules,
	pub confirm: KeymapRules,
	pub help:    KeymapRules,
	pub cmp:     KeymapRules,
}

impl Keymap {
	#[inline]
	pub fn get(&self, layer: Layer) -> &[Chord] {
		match layer {
			Layer::App => &[],
			Layer::Mgr => &self.mgr,
			Layer::Tasks => &self.tasks,
			Layer::Spot => &self.spot,
			Layer::Pick => &self.pick,
			Layer::Input => &self.input,
			Layer::Confirm => &self.confirm,
			Layer::Help => &self.help,
			Layer::Cmp => &self.cmp,
			Layer::Which => &[],
		}
	}
}

impl Keymap {
	pub(crate) fn reshape(self) -> Result<Self> {
		Ok(Self {
			mgr:     self.mgr.reshape(Layer::Mgr)?,
			tasks:   self.tasks.reshape(Layer::Tasks)?,
			spot:    self.spot.reshape(Layer::Spot)?,
			pick:    self.pick.reshape(Layer::Pick)?,
			input:   self.input.reshape(Layer::Input)?,
			confirm: self.confirm.reshape(Layer::Confirm)?,
			help:    self.help.reshape(Layer::Help)?,
			cmp:     self.cmp.reshape(Layer::Cmp)?,
		})
	}
}
