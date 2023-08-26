use std::{env, fs, path::{Path, PathBuf}, process, time::{self, SystemTime}};

use clap::{command, Parser};
use md5::{Digest, Md5};
use shared::absolute_path;

use crate::Xdg;

#[derive(Debug)]
pub struct Boot {
	pub cwd:       PathBuf,
	pub cache_dir: PathBuf,
	pub state_dir: PathBuf,

	pub cwd_file:     Option<PathBuf>,
	pub chooser_file: Option<PathBuf>,
}

#[derive(Debug, Parser)]
#[command(name = "yazi")]
#[command(version = "0.1.3")]
struct Args {
	/// Set the current working directory
	#[arg(long, short)]
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

		let cwd = args
			.cwd
			.map(|p| futures::executor::block_on(absolute_path(p)))
			.and_then(|p| p.is_dir().then_some(p))
			.or_else(|| env::current_dir().ok());

		let boot = Self {
			cwd:       cwd.unwrap_or("/".into()),
			cache_dir: env::temp_dir().join("yazi"),
			state_dir: Xdg::state_dir().unwrap(),

			cwd_file:     args.cwd_file,
			chooser_file: args.chooser_file,
		};

		if !boot.cache_dir.is_dir() {
			fs::create_dir(&boot.cache_dir).unwrap();
		}
		if !boot.state_dir.is_dir() {
			fs::create_dir_all(&boot.state_dir).unwrap();
		}

		if args.clear_cache {
			println!("Clearing cache directory: {:?}", boot.cache_dir);
			fs::remove_dir_all(&boot.cache_dir).unwrap();
			process::exit(0);
		}

		boot
	}
}

impl Boot {
	#[inline]
	pub fn cache(&self, path: &Path, skip: usize) -> PathBuf {
		self
			.cache_dir
			.join(format!("{:x}", Md5::new_with_prefix(format!("{:?}///{}", path, skip)).finalize()))
	}

	#[inline]
	pub fn tmpfile(&self, prefix: &str) -> PathBuf {
		let nanos = SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_nanos();
		self.cache_dir.join(format!("{prefix}-{}", nanos / 1000))
	}
}
