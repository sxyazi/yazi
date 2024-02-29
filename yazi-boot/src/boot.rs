use std::{ffi::OsString, path::{Path, PathBuf}, process};

use clap::Parser;
use serde::Serialize;
use yazi_config::PREVIEW;
use yazi_shared::{fs::{current_cwd, expand_path}, Xdg};

use super::Args;
use crate::ARGS;

#[derive(Debug, Serialize)]
pub struct Boot {
	pub cwd:  PathBuf,
	pub file: Option<OsString>,

	pub config_dir: PathBuf,
	pub flavor_dir: PathBuf,
	pub plugin_dir: PathBuf,
}

impl Boot {
	fn parse_entry(entry: Option<&Path>) -> (PathBuf, Option<OsString>) {
		let entry = match entry {
			Some(p) => expand_path(p),
			None => return (current_cwd().unwrap(), None),
		};

		let parent = entry.parent();
		if parent.is_none() || entry.is_dir() {
			return (entry, None);
		}

		(parent.unwrap().to_owned(), Some(entry.file_name().unwrap().to_owned()))
	}
}

impl Default for Boot {
	fn default() -> Self {
		let config_dir = Xdg::config_dir();
		let (cwd, file) = Self::parse_entry(ARGS.entry.as_deref());

		let boot = Self {
			cwd,
			file,

			flavor_dir: config_dir.join("flavors"),
			plugin_dir: config_dir.join("plugins"),
			config_dir,
		};

		std::fs::create_dir_all(&boot.flavor_dir).expect("Failed to create flavor directory");
		std::fs::create_dir_all(&boot.plugin_dir).expect("Failed to create plugin directory");
		boot
	}
}

impl Default for Args {
	fn default() -> Self {
		let args = Self::parse();

		if args.version {
			println!(
				"yazi {} ({} {})",
				env!("CARGO_PKG_VERSION"),
				env!("VERGEN_GIT_SHA"),
				env!("VERGEN_BUILD_DATE")
			);
			process::exit(0);
		}

		if args.clear_cache {
			if PREVIEW.cache_dir == Xdg::cache_dir() {
				println!("Clearing cache directory: \n{:?}", PREVIEW.cache_dir);
				std::fs::remove_dir_all(&PREVIEW.cache_dir).unwrap();
			} else {
				println!(
					"You've changed the default cache directory, for your data's safety, please clear it manually: \n{:?}",
					PREVIEW.cache_dir
				);
			}
			process::exit(0);
		}

		args
	}
}
