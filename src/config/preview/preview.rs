use std::fs;

use serde::Deserialize;
use xdg::BaseDirectories;

#[derive(Debug, Deserialize)]
pub struct Preview {
	pub tab_size: u32,

	pub max_width:  u32,
	pub max_height: u32,
}

impl Preview {
	pub fn new() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			preview: Preview,
		}

		let path = BaseDirectories::new().unwrap().get_config_file("yazi/yazi.toml");
		let outer: Outer = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
		outer.preview
	}
}
