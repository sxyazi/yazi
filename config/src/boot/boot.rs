use std::{env, fs, path::PathBuf};

use clap::Parser;

#[derive(Debug)]
pub struct Boot {
	pub cwd:       PathBuf,
	pub cwd_file:  Option<PathBuf>,
	pub cache_dir: PathBuf,
	pub state_dir: PathBuf,
}

#[derive(Debug, Parser)]
struct Args {
	/// Write the cwd on exit to this file
	#[arg(long)]
	cwd_file: Option<PathBuf>,
}

impl Default for Boot {
	fn default() -> Self {
		let args = Args::parse();
		let boot = Self {
			cwd:       env::current_dir().unwrap_or("/".into()),
			cwd_file:  args.cwd_file,
			cache_dir: env::temp_dir().join("yazi"),
			state_dir: xdg::BaseDirectories::with_prefix("yazi").unwrap().get_state_home(),
		};

		if !boot.cache_dir.is_dir() {
			fs::create_dir(&boot.cache_dir).unwrap();
		}
		if !boot.state_dir.is_dir() {
			fs::create_dir_all(&boot.state_dir).unwrap();
		}

		boot
	}
}
