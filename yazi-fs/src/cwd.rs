use std::{env::{current_dir, set_current_dir}, ops::Deref, path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}};

use arc_swap::ArcSwap;
use yazi_shared::{RoCell, url::Url};

pub static CWD: RoCell<Cwd> = RoCell::new();

pub struct Cwd(ArcSwap<Url>);

impl Deref for Cwd {
	type Target = ArcSwap<Url>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Default for Cwd {
	fn default() -> Self {
		let p = std::env::var_os("PWD")
			.map(PathBuf::from)
			.filter(|p| p.is_absolute())
			.or_else(|| current_dir().ok())
			.expect("failed to get current working directory");

		Self(ArcSwap::new(Arc::new(Url::from(p))))
	}
}

impl Cwd {
	pub fn set(&self, url: &Url) -> bool {
		if self.load().as_ref() == url {
			return false;
		}

		self.store(Arc::new(url.clone()));
		if let Some(p) = url.as_path() {
			unsafe { std::env::set_var("PWD", p) };
			Self::sync_cwd();
		}

		true
	}

	fn sync_cwd() {
		static SYNCING: AtomicBool = AtomicBool::new(false);
		if SYNCING.swap(true, Ordering::Relaxed) {
			return;
		}

		tokio::task::spawn_blocking(move || {
			if let Some(p) = CWD.load().as_path() {
				_ = set_current_dir(p);
			}

			let cur = current_dir().unwrap_or_default();
			SYNCING.store(false, Ordering::Relaxed);

			if let Some(p) = CWD.load().as_path()
				&& cur != p
			{
				set_current_dir(p).ok();
			}
		});
	}
}
