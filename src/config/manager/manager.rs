use std::{env, fs, path::PathBuf};

use serde::Deserialize;
use xdg::BaseDirectories;

use super::SortBy;

#[derive(Deserialize, Debug)]
pub struct Manager {
	#[serde(skip)]
	pub cwd:   PathBuf,
	#[serde(skip)]
	pub cache: PathBuf,

	// Sorting
	pub sort_by:      SortBy,
	pub sort_reverse: bool,

	// Display
	pub show_hidden: bool,
}

impl Manager {
	pub fn new() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			manager: Manager,
		}

		let path = BaseDirectories::new().unwrap().get_config_file("yazi/yazi.toml");
		let mut manager = toml::from_str::<Outer>(&fs::read_to_string(path).unwrap()).unwrap().manager;

		manager.cwd = env::current_dir().unwrap_or("/".into());
		manager.cache = "/tmp/yazi".into();
		if !manager.cache.is_dir() {
			fs::create_dir(&manager.cache).unwrap();
		}

		manager
	}
}
