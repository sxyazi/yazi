use std::{env::{current_dir, set_current_dir}, ops::Deref, path::PathBuf, sync::{Arc, atomic::{self, AtomicBool}}};

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
		unsafe { std::env::set_var("PWD", url) };

		Self::sync_cwd();
		true
	}

	fn sync_cwd() {
		static SYNCING: AtomicBool = AtomicBool::new(false);
		if SYNCING.swap(true, atomic::Ordering::Relaxed) {
			return;
		}

		tokio::task::spawn_blocking(move || {
			_ = set_current_dir(CWD.load().as_ref());
			let p = current_dir().unwrap_or_default();

			SYNCING.store(false, atomic::Ordering::Relaxed);
			if p != CWD.load().as_path() {
				set_current_dir(CWD.load().as_ref()).ok();
			}
		});
	}
}
