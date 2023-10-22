use std::fmt::{self, Display};

use serde::{Deserialize, Deserializer};

use super::Control;
use crate::MERGED_KEYMAP;

#[derive(Debug)]
pub struct Keymap {
	pub manager: Vec<Control>,
	pub tasks:   Vec<Control>,
	pub select:  Vec<Control>,
	pub input:   Vec<Control>,
	pub help:    Vec<Control>,
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
			help:    Inner,
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
			help:    shadow.help.keymap,
		})
	}
}

impl Default for Keymap {
	fn default() -> Self { toml::from_str(&MERGED_KEYMAP).unwrap() }
}

impl Keymap {
	#[inline]
	pub fn get(&self, layer: KeymapLayer) -> &Vec<Control> {
		match layer {
			KeymapLayer::Manager => &self.manager,
			KeymapLayer::Tasks => &self.tasks,
			KeymapLayer::Select => &self.select,
			KeymapLayer::Input => &self.input,
			KeymapLayer::Help => &self.help,
			KeymapLayer::Which => unreachable!(),
		}
	}
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub enum KeymapLayer {
	#[default]
	Manager,
	Tasks,
	Select,
	Input,
	Help,
	Which,
}

impl Display for KeymapLayer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			KeymapLayer::Manager => write!(f, "manager"),
			KeymapLayer::Tasks => write!(f, "tasks"),
			KeymapLayer::Select => write!(f, "select"),
			KeymapLayer::Input => write!(f, "input"),
			KeymapLayer::Help => write!(f, "help"),
			KeymapLayer::Which => write!(f, "which"),
		}
	}
}
