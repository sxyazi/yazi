use std::sync::OnceLock;

use tokio::sync::Notify;

#[derive(Debug)]
pub struct CompletionCell<T> {
	value: OnceLock<T>,
	ready: Notify,
}

impl<T> Default for CompletionCell<T> {
	fn default() -> Self { Self { value: OnceLock::new(), ready: Notify::new() } }
}

impl<T> CompletionCell<T> {
	pub fn get(&self) -> Option<&T> { self.value.get() }

	pub fn get_or_init<F>(&self, f: F) -> &T
	where
		F: FnOnce() -> T,
	{
		if let Some(value) = self.value.get() {
			return value;
		}

		let value = self.value.get_or_init(f);
		self.ready.notify_waiters();
		value
	}

	pub fn set(&self, value: T) -> Result<(), T> {
		self.value.set(value)?;
		self.ready.notify_waiters();
		Ok(())
	}

	pub async fn wait(&self) -> &T {
		loop {
			if let Some(value) = self.value.get() {
				return value;
			}

			let ready = self.ready.notified();
			if let Some(value) = self.value.get() {
				return value;
			}

			ready.await;
		}
	}
}
