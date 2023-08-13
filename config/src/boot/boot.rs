use std::{env, fs, path::{Path, PathBuf}, time::{self, SystemTime}};

use clap::Parser;
use md5::{Digest, Md5};

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

impl Boot {
	#[inline]
	pub fn cache(&self, path: &Path) -> PathBuf {
		self
			.cache_dir
			.join(format!("{:x}", Md5::new_with_prefix(path.to_string_lossy().as_bytes()).finalize()))
	}

	#[inline]
	pub fn tmpfile(&self) -> PathBuf {
		let nanos = SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_nanos();
		self.cache_dir.join(format!("{:x}", Md5::new_with_prefix(nanos.to_le_bytes()).finalize()))
	}
}
