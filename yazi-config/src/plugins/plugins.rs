use std::path::PathBuf;

use serde::Deserialize;
use validator::Validate;
use yazi_shared::fs::expand_path;

use crate::MERGED_YAZI;

#[derive(Debug, Deserialize, Validate)]
pub struct Plugins {
	pub preload: Vec<PathBuf>,
}

impl Default for Plugins {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			plugins: Plugins,
		}

		let mut plugins = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().plugins;

		plugins.preload.iter_mut().for_each(|p| {
			*p = expand_path(&p);
		});

		plugins
	}
}
