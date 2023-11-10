use std::{env, ffi::OsString, fs, path::PathBuf, process};

use clap::Parser;
use yazi_shared::expand_path;

use super::cli::Args;
use crate::{Xdg, PREVIEW};

#[derive(Debug)]
pub struct Boot {
	pub cwd:  PathBuf,
	pub file: Option<OsString>,

	pub state_dir: PathBuf,

	pub cwd_file:     Option<PathBuf>,
	pub chooser_file: Option<PathBuf>,
}

impl Boot {
	fn parse_entry(entry: Option<PathBuf>) -> (PathBuf, Option<OsString>) {
		let Some(entry) = entry else {
			return (env::current_dir().unwrap(), None);
		};

		let entry = expand_path(entry);
		let parent = entry.parent();
		if parent.is_none() || entry.is_dir() {
			return (entry, None);
		}

		return (parent.unwrap().to_owned(), Some(entry.file_name().unwrap().to_owned()));
	}
}

impl Default for Boot {
	fn default() -> Self {
		let args = Args::parse();
		let (cwd, file) = Self::parse_entry(args.entry);

		let boot = Self {
			cwd,
			file,

			state_dir: Xdg::state_dir().unwrap(),

			cwd_file: args.cwd_file,
			chooser_file: args.chooser_file,
		};

		if !boot.state_dir.is_dir() {
			fs::create_dir_all(&boot.state_dir).unwrap();
		}
		if !PREVIEW.cache_dir.is_dir() {
			fs::create_dir(&PREVIEW.cache_dir).unwrap();
		}

		if args.clear_cache {
			if PREVIEW.cache_dir == Xdg::cache_dir() {
				println!("Clearing cache directory: \n{:?}", PREVIEW.cache_dir);
				fs::remove_dir_all(&PREVIEW.cache_dir).unwrap();
			} else {
				println!(
					"You've changed the default cache directory, for your data's safety, please clear it manually: \n{:?}",
					PREVIEW.cache_dir
				);
			}
			process::exit(0);
		}

		boot
	}
}
