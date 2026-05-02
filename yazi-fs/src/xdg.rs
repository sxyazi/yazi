use std::{env, path::PathBuf, sync::OnceLock};

use yazi_macro::unix_either;

pub struct Xdg;

impl Xdg {
	pub(super) fn load() {
		Self::config_dir();
		Self::cache_dir();
		Self::state_dir();
		Self::runtime_dir();
		Self::temp_dir();
	}

	pub fn config_dir() -> &'static PathBuf {
		static ONCE: OnceLock<PathBuf> = OnceLock::new();
		ONCE.get_or_init(Self::load_config_dir)
	}

	fn load_config_dir() -> PathBuf {
		if let Some(p) = env::var_os("YAZI_CONFIG_HOME").map(PathBuf::from)
			&& p.is_absolute()
		{
			return p;
		}

		#[cfg(windows)]
		{
			dirs::config_dir().map(|p| p.join("yazi\\config")).expect("Failed to get config directory")
		}
		#[cfg(unix)]
		{
			env::var_os("XDG_CONFIG_HOME")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.map(|p| p.join("yazi"))
				.or_else(|| dirs::home_dir().map(|h| h.join(".config/yazi")))
				.expect("Failed to get config directory")
		}
	}

	pub fn cache_dir() -> &'static PathBuf {
		static ONCE: OnceLock<PathBuf> = OnceLock::new();
		ONCE.get_or_init(Self::load_cache_dir)
	}

	fn load_cache_dir() -> PathBuf {
		#[cfg(windows)]
		{
			dirs::cache_dir().map(|p| p.join("yazi")).expect("Failed to get cache directory")
		}
		#[cfg(unix)]
		{
			env::var_os("XDG_CACHE_HOME")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.map(|p| p.join("yazi"))
				.or_else(|| dirs::home_dir().map(|h| h.join(".cache/yazi")))
				.expect("Failed to get cache directory")
		}
	}

	pub fn state_dir() -> &'static PathBuf {
		static ONCE: OnceLock<PathBuf> = OnceLock::new();
		ONCE.get_or_init(Self::load_state_dir)
	}

	fn load_state_dir() -> PathBuf {
		#[cfg(windows)]
		{
			dirs::data_dir().map(|p| p.join("yazi\\state")).expect("Failed to get state directory")
		}
		#[cfg(unix)]
		{
			env::var_os("XDG_STATE_HOME")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.map(|p| p.join("yazi"))
				.or_else(|| dirs::home_dir().map(|h| h.join(".local/state/yazi")))
				.expect("Failed to get state directory")
		}
	}

	pub fn runtime_dir() -> &'static PathBuf {
		static ONCE: OnceLock<PathBuf> = OnceLock::new();
		ONCE.get_or_init(Self::load_runtime_dir)
	}

	fn load_runtime_dir() -> PathBuf {
		let mut p = env::var_os("XDG_RUNTIME_DIR")
			.map(PathBuf::from)
			.filter(|p| p.is_absolute())
			.unwrap_or_else(|| env::temp_dir());

		let uid = unix_either!(
			{
				use uzers::Users;
				yazi_shared::USERS_CACHE.get_current_uid()
			},
			0
		);

		p.push(format!("yazi+{uid}"));
		p
	}

	pub fn temp_dir() -> &'static PathBuf {
		static ONCE: OnceLock<PathBuf> = OnceLock::new();
		ONCE.get_or_init(Self::load_temp_dir)
	}

	fn load_temp_dir() -> PathBuf {
		let mut p = env::temp_dir();
		assert!(p.is_absolute(), "Temporary directory path is not absolute");

		let uid = unix_either!(
			{
				use uzers::Users;
				yazi_shared::USERS_CACHE.get_current_uid()
			},
			0
		);

		p.push(format!("yazi-{uid}"));
		p
	}
}
