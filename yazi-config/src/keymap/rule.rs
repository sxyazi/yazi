use std::{collections::HashSet, ops::Deref};

use anyhow::Result;
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_shared::Layer;

use super::Chord;
use crate::{Preset, keymap::Key};

#[derive(Default, Deserialize, DeserializeOver2)]
pub struct KeymapRule {
	pub keymap:     Vec<Chord>,
	#[serde(default)]
	prepend_keymap: Vec<Chord>,
	#[serde(default)]
	append_keymap:  Vec<Chord>,
}

impl Deref for KeymapRule {
	type Target = Vec<Chord>;

	fn deref(&self) -> &Self::Target { &self.keymap }
}

impl KeymapRule {
	pub(crate) fn reshape(self, layer: Layer) -> Result<Self> {
		#[inline]
		fn on(Chord { on, .. }: &Chord) -> [Key; 2] {
			[on.first().copied().unwrap_or_default(), on.get(1).copied().unwrap_or_default()]
		}

		let a_seen: HashSet<_> = self.prepend_keymap.iter().map(on).collect();
		let b_seen: HashSet<_> = self.keymap.iter().map(on).collect();

		let keymap = Preset::mix(
			self.prepend_keymap,
			self.keymap.into_iter().filter(|v| !a_seen.contains(&on(v))),
			self.append_keymap.into_iter().filter(|v| !b_seen.contains(&on(v))),
		)
		.filter(|chord| !chord.noop())
		.map(|chord| chord.with_layer(layer))
		.collect();

		Ok(Self { keymap, ..Default::default() })
	}
}
