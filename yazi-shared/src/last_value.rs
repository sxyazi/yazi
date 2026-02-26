use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::Notify;

#[derive(Clone, Debug, Default)]
pub struct LastValue<T> {
	inner: Arc<(Notify, Mutex<Option<T>>)>,
}

impl<T> LastValue<T> {
	pub fn set(&self, data: T) {
		*self.inner.1.lock() = Some(data);
		self.inner.0.notify_waiters();
	}

	pub async fn get(&self) -> T {
		loop {
			let notified = self.inner.0.notified();
			if let Some(data) = self.inner.1.lock().take() {
				return data;
			}

			notified.await;
		}
	}
}
