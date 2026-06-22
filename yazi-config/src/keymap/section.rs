use std::ops::Deref;

use hashbrown::HashSet;
use mlua::{UserData, UserDataFields};
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_shim::{mlua::UserDataFieldsExt, toml::DeserializeOverHook};

use super::{Chord, Chords, Key};
use crate::{keymap::ChordArc, mix};

#[derive(Default, Deserialize, DeserializeOver2)]
pub struct KeymapSection {
	keymap:         Chords,
	#[serde(default)]
	prepend_keymap: Vec<Chord>,
	#[serde(default)]
	append_keymap:  Vec<Chord>,
}

impl Deref for KeymapSection {
	type Target = Chords;

	fn deref(&self) -> &Self::Target { &self.keymap }
}

impl DeserializeOverHook for KeymapSection {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		#[inline]
		fn on<T: AsRef<Chord>>(chord: T) -> [Key; 2] {
			let c = chord.as_ref();
			[c.on.first().copied().unwrap_or_default(), c.on.get(1).copied().unwrap_or_default()]
		}

		let keymap = self.keymap.unwrap_unchecked();
		let a_seen: HashSet<_> = self.prepend_keymap.iter().map(on).collect();
		let b_seen: HashSet<_> = keymap.iter().map(on).collect();

		let keymap: Vec<ChordArc> = mix(
			self.prepend_keymap,
			keymap.into_iter().filter(|v| !a_seen.contains(&on(v))),
			self.append_keymap.into_iter().filter(|v| !b_seen.contains(&on(v))),
		);

		Ok(Self { keymap: keymap.into(), ..Default::default() })
	}
}

impl UserData for &'static KeymapSection {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("rules", |_, me| Ok(&me.keymap));
	}
}
