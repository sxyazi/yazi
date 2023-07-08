use std::fs;

use serde::Deserialize;
use xdg::BaseDirectories;

use super::{Exec, Key};

#[derive(Deserialize, Debug)]
pub struct Single {
	pub on:   Vec<Key>,
	#[serde(deserialize_with = "Exec::deserialize")]
	pub exec: Vec<Exec>,
}

#[derive(Deserialize, Debug)]
pub struct Keymap {
	pub manager: Vec<Single>,
	pub tasks:   Vec<Single>,
	pub input:   Vec<Single>,
}

impl Keymap {
	pub fn new() -> Self {
		#[derive(Deserialize)]
		struct Inner {
			keymap: Vec<Single>,
		}

		#[derive(Deserialize)]
		struct All {
			manager: Inner,
			tasks:   Inner,
			input:   Inner,
		}

		let path = BaseDirectories::new().unwrap().get_config_file("yazi/keymap.toml");

		let all: All = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
		Self { manager: all.manager.keymap, tasks: all.tasks.keymap, input: all.input.keymap }
	}
}
