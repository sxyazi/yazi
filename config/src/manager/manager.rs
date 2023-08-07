use std::{env, fs, path::PathBuf};

use serde::Deserialize;

use super::SortBy;
use crate::MERGED_YAZI;

#[derive(Debug, Deserialize)]
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

		let mut manager = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().manager;

		manager.cwd = env::current_dir().unwrap_or("/".into());
		manager.cache = env::temp_dir().join("yazi");
		if !manager.cache.is_dir() {
			fs::create_dir(&manager.cache).unwrap();
		}

		manager
	}
}
