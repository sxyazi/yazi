use std::{ops::Deref, path::PathBuf, sync::Arc};

use arc_swap::ArcSwap;
use yazi_shared::{RoCell, url::Url};

pub static CWD: RoCell<Cwd> = RoCell::new();

pub struct Cwd {
	inner: ArcSwap<Url>,
}

impl Deref for Cwd {
	type Target = ArcSwap<Url>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Default for Cwd {
	fn default() -> Self {
		let p = std::env::var_os("PWD")
			.map(PathBuf::from)
			.filter(|p| p.is_absolute())
			.or_else(|| std::env::current_dir().ok())
			.expect("failed to get current working directory");

		Self { inner: ArcSwap::new(Arc::new(Url::from(p))) }
	}
}

impl Cwd {
	pub fn set(&self, url: &Url) {
		self.inner.store(Arc::new(url.clone()));
		std::env::set_var("PWD", self.inner.load().as_ref());
	}
}
