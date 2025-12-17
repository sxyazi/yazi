use std::sync::{Arc, atomic::{AtomicU8, Ordering}};

use tokio::sync::Notify;

#[derive(Clone, Debug, Default)]
pub struct CompletionToken {
	inner: Arc<(AtomicU8, Notify)>,
}

impl CompletionToken {
	pub fn complete(&self, success: bool) {
		let new = if success { 1 } else { 2 };
		self.inner.0.compare_exchange(0, new, Ordering::Relaxed, Ordering::Relaxed).ok();
		self.inner.1.notify_waiters();
	}

	pub fn completed(&self) -> Option<bool> {
		let state = self.inner.0.load(Ordering::Relaxed);
		if state == 0 { None } else { Some(state == 1) }
	}

	pub async fn future(&self) -> bool {
		loop {
			if let Some(state) = self.completed() {
				return state;
			}

			let notified = self.inner.1.notified();
			if let Some(state) = self.completed() {
				return state;
			}

			notified.await;
		}
	}
}
