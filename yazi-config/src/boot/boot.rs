use std::{ffi::OsString, fs, path::{Path, PathBuf}, process};

use clap::Parser;
use serde::Serialize;
use yazi_shared::fs::{current_cwd, expand_path};

use super::Args;
use crate::{Xdg, ARGS};

#[derive(Debug, Serialize)]
pub struct Boot {
	pub cwd:  PathBuf,
	pub file: Option<OsString>,

	pub config_dir: PathBuf,
	pub plugin_dir: PathBuf,
	pub state_dir:  PathBuf,
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

		return (parent.unwrap().to_owned(), Some(entry.file_name().unwrap().to_owned()));
	}
}

impl Default for Boot {
	fn default() -> Self {
		let (cwd, file) = Self::parse_entry(ARGS.entry.as_deref());
		let boot = Self {
			cwd,
			file,

			config_dir: Xdg::config_dir().unwrap(),
			plugin_dir: Xdg::plugin_dir().unwrap(),
			state_dir: Xdg::state_dir().unwrap(),
		};

		if !boot.state_dir.is_dir() {
			fs::create_dir_all(&boot.state_dir).unwrap();
		}

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

		args
	}
}
