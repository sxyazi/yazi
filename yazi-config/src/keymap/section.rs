use std::ops::Deref;

use hashbrown::HashSet;
use mlua::{UserData, UserDataFields};
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_shared::Layer;
use yazi_shim::{mlua::UserDataFieldsExt, toml::DeserializeOverHook};

use super::{Key, Chord, Chords, chords::layer_default};
use crate::{keymap::ChordArc, mix};

#[derive(Default, Deserialize, DeserializeOver2)]
pub struct KeymapSection<const L: u8 = { Layer::Null as u8 }> {
	keymap:         Chords<L>,
	#[serde(default)]
	prepend_keymap: Vec<Chord<L>>,
	#[serde(default)]
	append_keymap:  Vec<Chord<L>>,
	#[serde(default = "layer_default::<L>")]
	layer:          Layer,
}

impl<const L: u8> Deref for KeymapSection<L> {
	type Target = Chords<L>;

	fn deref(&self) -> &Self::Target { &self.keymap }
}

impl<const L: u8> KeymapSection<L> {
	pub fn as_erased<const M: u8>(&self) -> &KeymapSection<M> {
		unsafe { &*(self as *const Self as *const KeymapSection<M>) }
	}
}

impl<const L: u8> DeserializeOverHook for KeymapSection<L> {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		#[inline]
		fn on<const L: u8>(Chord { on, .. }: &Chord<L>) -> [Key; 2] {
			[on.first().copied().unwrap_or_default(), on.get(1).copied().unwrap_or_default()]
		}

		let keymap = self.keymap.unwrap_unchecked();
		let a_seen: HashSet<_> = self.prepend_keymap.iter().map(on).collect();
		let b_seen: HashSet<_> = keymap.iter().map(|c| on(c)).collect();

		let keymap: Vec<ChordArc<L>> = mix(
			self.prepend_keymap,
			keymap.into_iter().filter(|v| !a_seen.contains(&on(v))),
			self.append_keymap.into_iter().filter(|v| !b_seen.contains(&on(v))),
		);

		Ok(Self { keymap: keymap.into(), layer: self.layer, ..Default::default() })
	}
}

impl UserData for &'static KeymapSection {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("rules", |_, me| Ok(&me.keymap));
	}
}
