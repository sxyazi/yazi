use std::ops::Deref;

use hashbrown::HashSet;
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_shim::toml::DeserializeOverHook;

use super::{Chord, Key};
use crate::mix;

#[derive(Default, Deserialize, DeserializeOver2)]
pub struct KeymapRules<const L: u8> {
	keymap:         Vec<Chord<L>>,
	#[serde(default)]
	prepend_keymap: Vec<Chord<L>>,
	#[serde(default)]
	append_keymap:  Vec<Chord<L>>,
}

impl<const L: u8> Deref for KeymapRules<L> {
	type Target = [Chord];

	fn deref(&self) -> &Self::Target { self.as_erased_slice() }
}

impl<const L: u8> KeymapRules<L> {
	pub(super) fn as_erased_slice(&self) -> &[Chord] {
		// Safety: `Chord<L>` only changes deserialization behavior; the const parameter
		// does not participate in layout, so a shared slice can be reinterpreted as
		// the default `Chord` view.
		unsafe { &*(self.keymap.as_slice() as *const [Chord<L>] as *const [Chord]) }
	}
}

impl<const L: u8> DeserializeOverHook for KeymapRules<L> {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		#[inline]
		fn on<const L: u8>(Chord { on, .. }: &Chord<L>) -> [Key; 2] {
			[on.first().copied().unwrap_or_default(), on.get(1).copied().unwrap_or_default()]
		}

		let a_seen: HashSet<_> = self.prepend_keymap.iter().map(on).collect();
		let b_seen: HashSet<_> = self.keymap.iter().map(on).collect();

		let keymap = mix(
			self.prepend_keymap,
			self.keymap.into_iter().filter(|v| !a_seen.contains(&on(v))),
			self.append_keymap.into_iter().filter(|v| !b_seen.contains(&on(v))),
		);

		Ok(Self { keymap, ..Default::default() })
	}
}
