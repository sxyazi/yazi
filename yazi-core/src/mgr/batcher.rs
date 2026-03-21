use std::{path::{Path, PathBuf}, sync::Arc};

use parking_lot::Mutex;
use yazi_shared::url::AsUrl;

#[derive(Clone, Default)]
pub struct Batcher {
	pending: Arc<Mutex<(PathBuf, Option<bool>)>>,
}

impl Batcher {
	pub fn prime<T>(&self, target: T)
	where
		T: Into<PathBuf>,
	{
		*self.pending.lock() = (target.into(), None);
	}

	pub fn drain(&self, target: &Path) -> Option<bool> {
		let mut pending = self.pending.lock();
		if pending.0 != target {
			return None;
		}

		pending.0 = PathBuf::default();
		pending.1.take()
	}

	pub fn decide<T>(&self, target: T, decision: bool)
	where
		T: AsUrl,
	{
		let mut pending = self.pending.lock();
		if target.as_url() == pending.0.as_url() {
			pending.1 = Some(decision);
		}
	}
}
