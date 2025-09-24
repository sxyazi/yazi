use std::{env::{current_dir, set_current_dir}, ops::Deref, path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}};

use arc_swap::ArcSwap;
use yazi_shared::{RoCell, url::UrlBuf};

use crate::provider;

pub static CWD: RoCell<Cwd> = RoCell::new();

pub struct Cwd(ArcSwap<UrlBuf>);

impl Deref for Cwd {
	type Target = ArcSwap<UrlBuf>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Default for Cwd {
	fn default() -> Self {
		let p = std::env::var_os("PWD")
			.map(PathBuf::from)
			.filter(|p| p.is_absolute())
			.or_else(|| current_dir().ok())
			.expect("failed to get current working directory");

		Self(ArcSwap::new(Arc::new(UrlBuf::from(p))))
	}
}

impl Cwd {
	pub fn path(&self) -> PathBuf {
		let url = self.0.load();
		provider::cache(url.as_ref()).unwrap_or_else(|| url.loc.to_path())
	}

	pub fn set(&self, url: &UrlBuf) -> bool {
		if !url.is_absolute() {
			return false;
		} else if self.0.load().as_ref() == url {
			return false;
		}

		self.0.store(Arc::new(url.clone()));
		Self::sync_cwd();

		true
	}

	fn sync_cwd() {
		static SYNCING: AtomicBool = AtomicBool::new(false);
		if SYNCING.swap(true, Ordering::Relaxed) {
			return;
		}

		tokio::task::spawn_blocking(move || {
			let path = CWD.path();
			std::fs::create_dir_all(&path).ok();

			_ = set_current_dir(&path);
			let cur = current_dir().unwrap_or_default();

			unsafe { std::env::set_var("PWD", path) }
			SYNCING.store(false, Ordering::Relaxed);

			let path = CWD.path();
			if cur != path {
				std::fs::create_dir_all(&path).ok();
				set_current_dir(&path).ok();
				unsafe { std::env::set_var("PWD", path) }
			}
		});
	}
}
