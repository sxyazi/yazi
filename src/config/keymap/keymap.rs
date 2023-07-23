use serde::{Deserialize, Deserializer};

use super::{Exec, Key};
use crate::config::MERGED_KEYMAP;

#[derive(Clone, Debug, Deserialize)]
pub struct Control {
	pub on:   Vec<Key>,
	#[serde(deserialize_with = "Exec::deserialize")]
	pub exec: Vec<Exec>,
}

#[derive(Debug)]
pub struct Keymap {
	pub manager: Vec<Control>,
	pub tasks:   Vec<Control>,
	pub select:  Vec<Control>,
	pub input:   Vec<Control>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum KeymapLayer {
	Manager,
	Tasks,
	Select,
	Input,
	Which,
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
			select:  Inner,
			input:   Inner,
		}
		#[derive(Deserialize)]
		struct Inner {
			keymap: Vec<Control>,
		}

		let shadow = Shadow::deserialize(deserializer)?;
		Ok(Self {
			manager: shadow.manager.keymap,
			tasks:   shadow.tasks.keymap,
			select:  shadow.select.keymap,
			input:   shadow.input.keymap,
		})
	}
}

impl Keymap {
	pub fn new() -> Self { toml::from_str(&MERGED_KEYMAP).unwrap() }

	#[inline]
	pub fn get(&self, layer: KeymapLayer) -> &Vec<Control> {
		match layer {
			KeymapLayer::Manager => &self.manager,
			KeymapLayer::Tasks => &self.tasks,
			KeymapLayer::Select => &self.select,
			KeymapLayer::Input => &self.input,
			KeymapLayer::Which => unreachable!(),
		}
	}
}
