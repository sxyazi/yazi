use std::path::PathBuf;

pub(super) struct Xdg;

impl Xdg {
	pub(super) fn config_dir() -> Option<PathBuf> {
		#[cfg(target_os = "windows")]
		{
			dirs::config_dir().map(|p| p.join("yazi").join("config"))
		}
		#[cfg(not(target_os = "windows"))]
		{
			std::env::var_os("XDG_CONFIG_HOME")
				.map(PathBuf::from)
				.and_then(|p| p.is_absolute().then_some(p))
				.or_else(|| dirs::home_dir().map(|h| h.join(".config")))
				.map(|p| p.join("yazi"))
		}
	}

	pub(super) fn state_dir() -> Option<PathBuf> {
		#[cfg(target_os = "windows")]
		{
			dirs::data_dir().map(|p| p.join("yazi").join("state"))
		}
		#[cfg(not(target_os = "windows"))]
		{
			std::env::var_os("XDG_STATE_HOME")
				.map(PathBuf::from)
				.and_then(|p| p.is_absolute().then_some(p))
				.or_else(|| dirs::home_dir().map(|h| h.join(".local/state")))
				.map(|p| p.join("yazi"))
		}
	}
}
