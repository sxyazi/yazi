use std::{env, path::PathBuf};

use yazi_shared::fs::expand_path;

pub(super) struct Xdg;

impl Xdg {
	pub(super) fn config_dir() -> Option<PathBuf> {
		if let Some(s) = env::var_os("YAZI_CONFIG_HOME").filter(|s| !s.is_empty()) {
			return Some(expand_path(s));
		}

		#[cfg(windows)]
		{
			dirs::config_dir().map(|p| p.join("yazi").join("config"))
		}
		#[cfg(unix)]
		{
			env::var_os("XDG_CONFIG_HOME")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.or_else(|| dirs::home_dir().map(|h| h.join(".config")))
				.map(|p| p.join("yazi"))
		}
	}

	#[inline]
	pub(super) fn plugin_dir() -> Option<PathBuf> { Self::config_dir().map(|p| p.join("plugins")) }

	pub(super) fn state_dir() -> Option<PathBuf> {
		#[cfg(windows)]
		{
			dirs::data_dir().map(|p| p.join("yazi").join("state"))
		}
		#[cfg(unix)]
		{
			env::var_os("XDG_STATE_HOME")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.or_else(|| dirs::home_dir().map(|h| h.join(".local/state")))
				.map(|p| p.join("yazi"))
		}
	}

	#[inline]
	pub(super) fn cache_dir() -> PathBuf { env::temp_dir().join("yazi") }
}
