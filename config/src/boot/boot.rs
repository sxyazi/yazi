use std::{env, fs, path::{Path, PathBuf}, time::{self, SystemTime}};

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
			state_dir: xdg::BaseDirectories::with_prefix("yazi").unwrap().get_state_home(),

			cwd_file:     args.cwd_file,
			chooser_file: args.chooser_file,
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
		#[cfg(target_os = "windows")]
		let h = Md5::new_with_prefix(path.to_string_lossy().as_bytes());

		#[cfg(not(target_os = "windows"))]
		let h = {
			use std::os::unix::ffi::OsStrExt;
			Md5::new_with_prefix(path.as_os_str().as_bytes())
		};

		self.cache_dir.join(format!("{:x}", h.finalize()))
	}

	#[inline]
	pub fn tmpfile(&self, prefix: &str) -> PathBuf {
		let nanos = SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_nanos();
		self.cache_dir.join(format!("{prefix}-{}", nanos / 1000))
	}
}
