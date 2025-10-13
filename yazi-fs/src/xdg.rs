use std::{env, path::PathBuf, sync::OnceLock};

pub struct Xdg;

impl Xdg {
	pub fn config_dir() -> PathBuf {
		if let Some(p) = env::var_os("YAZI_CONFIG_HOME").map(PathBuf::from)
			&& p.is_absolute()
		{
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

	pub fn cache_dir() -> &'static PathBuf {
		static CACHE: OnceLock<PathBuf> = OnceLock::new();

		CACHE.get_or_init(|| {
			let mut p = env::temp_dir();
			assert!(p.is_absolute(), "Temp dir is not absolute");

			#[cfg(unix)]
			{
				use uzers::Users;
				p.push(format!("yazi-{}", yazi_shared::USERS_CACHE.get_current_uid()))
			}
			#[cfg(not(unix))]
			p.push("yazi");

			p
		})
	}
}
