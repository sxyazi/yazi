use std::{env, path::PathBuf};

use crate::expand_path;

pub struct Xdg;

impl Xdg {
	pub fn config_dir() -> PathBuf {
		if let Some(p) = env::var_os("YAZI_CONFIG_HOME").map(expand_path).filter(|p| p.is_absolute()) {
			return p;
		}

		#[cfg(windows)]
		{
			dirs::config_dir()
				.map(|p| p.join("yazi").join("config"))
				.expect("Failed to get config directory")
		}
		#[cfg(unix)]
		{
			env::var_os("XDG_CONFIG_HOME")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.or_else(|| dirs::home_dir().map(|h| h.join(".config")))
				.map(|p| p.join("yazi"))
				.expect("Failed to get config directory")
		}
	}

	pub fn state_dir() -> PathBuf {
		#[cfg(windows)]
		{
			dirs::data_dir().map(|p| p.join("yazi").join("state")).expect("Failed to get state directory")
		}
		#[cfg(unix)]
		{
			env::var_os("XDG_STATE_HOME")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.or_else(|| dirs::home_dir().map(|h| h.join(".local/state")))
				.map(|p| p.join("yazi"))
				.expect("Failed to get state directory")
		}
	}

	#[inline]
	pub fn cache_dir() -> PathBuf {
		#[cfg(unix)]
		let s = {
			use uzers::Users;
			format!("yazi-{}", yazi_shared::USERS_CACHE.get_current_uid())
		};

		#[cfg(windows)]
		let s = "yazi";

		env::temp_dir().join(s)
	}
}
