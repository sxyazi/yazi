use std::fs;

use serde::Deserialize;
use xdg::BaseDirectories;

#[derive(Deserialize, Debug)]
pub struct Manager {
	pub sort_by:      String,
	pub sort_reverse: bool,
}

#[derive(Deserialize, Debug)]
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
		toml::from_str::<Outer>(&fs::read_to_string(path).unwrap()).unwrap().preview
	}
}
