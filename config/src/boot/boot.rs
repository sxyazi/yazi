use std::{env, fs, path::PathBuf};

#[derive(Debug)]
pub struct Boot {
	pub cwd:       PathBuf,
	pub cache_dir: PathBuf,
	pub state_dir: PathBuf,
}

impl Default for Boot {
	fn default() -> Self {
		let boot = Self {
			cwd:       env::current_dir().unwrap_or("/".into()),
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
