use std::{path::{Path, PathBuf}, sync::Arc};

use hashbrown::HashMap;
use parking_lot::Mutex;
use yazi_shared::url::AsUrl;

#[derive(Clone, Default)]
pub struct Batcher {
	pending: Arc<Mutex<HashMap<PathBuf, Option<bool>>>>,
}

impl Batcher {
	pub fn prime<T>(&self, target: T)
	where
		T: Into<PathBuf>,
	{
		self.pending.lock().insert(target.into(), None);
	}

	pub fn drain(&self, target: &Path) -> Option<bool> {
		self.pending.lock().remove(target).flatten()
	}

	pub fn decide<T>(&self, target: T, decision: bool)
	where
		T: AsUrl,
	{
		let Some(path) = target.as_url().as_local() else { return };
		if let Some(value) = self.pending.lock().get_mut(path) {
			*value = value.or(Some(decision));
		}
	}
}
