use std::{env, fs, path::PathBuf, process};

use clap::{command, Parser};
use shared::absolute_path;

use crate::{Xdg, PREVIEW};

#[derive(Debug)]
pub struct Boot {
	pub cwd:       PathBuf,
	pub state_dir: PathBuf,

	pub cwd_file:     Option<PathBuf>,
	pub chooser_file: Option<PathBuf>,
}

#[derive(Debug, Parser)]
#[command(name = "yazi")]
#[command(version = "0.1.4")]
struct Args {
	// -- TODO: Deprecate this in v0.1.5
	/// Set the current working directory
	#[arg(long, short)]
	_cwd: Option<PathBuf>,

	/// Set the current working directory
	#[arg(index = 1)]
	cwd: Option<PathBuf>,

	/// Write the cwd on exit to this file
	#[arg(long)]
	cwd_file:     Option<PathBuf>,
	/// Write the selected files on open emitted by the chooser mode
	#[arg(long)]
	chooser_file: Option<PathBuf>,

	/// Clear the cache directory
	#[arg(long, action)]
	clear_cache: bool,
}

impl Default for Boot {
	fn default() -> Self {
		let args = Args::parse();

		// -- TODO: Deprecate this in v0.1.5
		let cwd = if args._cwd.is_some() {
			println!(
				"Warning: -c/--cwd is deprecated in v0.1.5, please use the positional argument instead: `yazi --cwd /path/to/dir` -> `yazi /path/to/dir`.\nSee https://github.com/sxyazi/yazi/issues/95 for more information."
			);
			args._cwd
		} else {
			args.cwd
		};
		// TODO: Deprecate this in v0.1.5 --

		let cwd = cwd.map(absolute_path).filter(|p| p.is_dir()).or_else(|| env::current_dir().ok());

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
