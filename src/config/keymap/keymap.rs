use serde::{Deserialize, Deserializer};

use super::{Exec, Key};
use crate::config::MERGED_KEYMAP;

#[derive(Debug, Deserialize)]
pub struct Single {
	pub on:   Vec<Key>,
	#[serde(deserialize_with = "Exec::deserialize")]
	pub exec: Vec<Exec>,
}

#[derive(Debug)]
pub struct Keymap {
	pub manager: Vec<Single>,
	pub tasks:   Vec<Single>,
	pub input:   Vec<Single>,
}

impl<'de> Deserialize<'de> for Keymap {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Shadow {
			manager: Inner,
			tasks:   Inner,
			input:   Inner,
		}
		#[derive(Deserialize)]
		struct Inner {
			keymap: Vec<Single>,
		}

		let shadow = Shadow::deserialize(deserializer)?;
		Ok(Self {
			manager: shadow.manager.keymap,
			tasks:   shadow.tasks.keymap,
			input:   shadow.input.keymap,
		})
	}
}

impl Keymap {
	pub fn new() -> Self { toml::from_str(&MERGED_KEYMAP).unwrap() }
}
