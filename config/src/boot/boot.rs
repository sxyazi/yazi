use std::{env, fs, path::PathBuf, process};

use clap::Parser;
use shared::expand_path;

use super::cli::Args;
use crate::{Xdg, PREVIEW};

#[derive(Debug)]
pub struct Boot {
	pub cwd:       PathBuf,
	pub state_dir: PathBuf,

	pub cwd_file:     Option<PathBuf>,
	pub chooser_file: Option<PathBuf>,
}

impl Default for Boot {
	fn default() -> Self {
		let args = Args::parse();

		let cwd = args.cwd.map(expand_path).filter(|p| p.is_dir()).or_else(|| env::current_dir().ok());

		let boot = Self {
			cwd:       cwd.unwrap_or("/".into()),
			state_dir: Xdg::state_dir().unwrap(),

			cwd_file:     args.cwd_file,
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
