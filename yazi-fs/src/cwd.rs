use std::{borrow::Cow, env::{current_dir, set_current_dir}, ops::Deref, path::{Path, PathBuf}, sync::{Arc, atomic::{AtomicBool, Ordering}}};

use arc_swap::ArcSwap;
use yazi_shared::{RoCell, url::{AsUrl, Url, UrlBuf, UrlLike}};

use crate::{FsUrl, Xdg};

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
		url.cache().unwrap_or_else(|| url.loc.to_path())
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

	pub fn ensure(url: Url) -> Cow<Path> {
		use std::io::ErrorKind::{AlreadyExists, NotADirectory, NotFound};

		let Some(cache) = url.cache() else {
			return url.loc.as_path().into();
		};

		if !matches!(std::fs::create_dir_all(&cache), Err(e) if e.kind() == NotADirectory || e.kind() == AlreadyExists)
		{
			return cache.into();
		}

		let count = cache.strip_prefix(Xdg::cache_dir()).expect("under cache dir").components().count();
		for n in (0..count).rev() {
			let mut it = cache.components();
			for _ in 0..n {
				it.next_back().unwrap();
			}
			match std::fs::remove_file(it.as_path()) {
				Ok(_) => break,
				Err(e) if e.kind() == NotFound => break,
				Err(_) => {}
			}
		}

		std::fs::create_dir_all(&cache).ok();
		cache.into()
	}

	fn sync_cwd() {
		static SYNCING: AtomicBool = AtomicBool::new(false);
		if SYNCING.swap(true, Ordering::Relaxed) {
			return;
		}

		tokio::task::spawn_blocking(move || {
			let cwd = CWD.load();
			let path = Self::ensure(cwd.as_url());

			_ = set_current_dir(&path);
			let cur = current_dir().unwrap_or_default();

			unsafe { std::env::set_var("PWD", path.as_ref()) }
			SYNCING.store(false, Ordering::Relaxed);

			let cwd = CWD.load();
			let path = Self::ensure(cwd.as_url());
			if cur != path {
				set_current_dir(&path).ok();
				unsafe { std::env::set_var("PWD", path.as_ref()) }
			}
		});
	}
}
