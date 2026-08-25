use std::sync::{Arc, atomic::{AtomicU8, Ordering}};

use tokio::sync::Notify;
use yazi_macro::impl_data_any;
use yazi_shared::{data::Data, id::Id};

use crate::TaskStatus;

#[derive(Clone, Debug)]
pub struct TaskHandle {
	pub id: Id,
	inner:  Arc<(AtomicU8, Notify)>,
}

impl_data_any!(TaskHandle);

impl From<TaskHandle> for Data {
	fn from(value: TaskHandle) -> Self { Self::Any(Box::new(value)) }
}

impl TaskHandle {
	pub(super) fn new(id: Id) -> Self {
		Self { id, inner: Arc::new((AtomicU8::new(TaskStatus::Pending as u8), Notify::new())) }
	}

	pub(crate) fn start(&self) {
		let result = self.inner.0.compare_exchange(
			TaskStatus::Pending as u8,
			TaskStatus::Started as u8,
			Ordering::Relaxed,
			Ordering::Relaxed,
		);

		if result.is_ok() {
			self.inner.1.notify_waiters();
		}
	}

	pub(crate) fn succeed(&self) {
		self.transition(TaskStatus::Succeeded, TaskStatus::is_finishable);
	}

	pub(crate) fn fail(&self) { self.transition(TaskStatus::Failed, TaskStatus::is_finishable); }

	pub(crate) fn cancel(&self) { self.transition(TaskStatus::Canceled, TaskStatus::is_cancelable); }

	pub(crate) fn status(&self) -> TaskStatus {
		TaskStatus::from_repr(self.inner.0.load(Ordering::Relaxed)).unwrap_or_default()
	}

	pub(crate) fn is_canceled(&self) -> bool { self.status().is_canceled() }

	pub async fn started(&self) -> bool {
		self.wait(|status| !status.is_pending()).await.has_started()
	}

	pub(crate) async fn finished(&self) -> TaskStatus { self.wait(TaskStatus::is_finished).await }

	pub async fn future(&self) -> bool { self.finished().await.is_succeeded() }

	fn transition(&self, status: TaskStatus, allowed: fn(TaskStatus) -> bool) {
		let result = self.inner.0.try_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
			TaskStatus::from_repr(old).is_some_and(allowed).then_some(status as u8)
		});

		if result.is_ok() {
			self.inner.1.notify_waiters();
		}
	}

	async fn wait(&self, ready: impl Fn(TaskStatus) -> bool) -> TaskStatus {
		loop {
			let status = self.status();
			if ready(status) {
				return status;
			}

			let notified = self.inner.1.notified();
			let status = self.status();
			if ready(status) {
				return status;
			}

			notified.await;
		}
	}
}
