use std::{cell::Cell, future::Future, rc::Rc, thread};

use compact_str::CompactString;
use scopeguard::{ScopeGuard, guard};
use tokio::task::{self, JoinHandle};
use yazi_shim::fs::Error;

use super::AsyncResult;

pub struct AsyncHandle<T> {
	handle:    JoinHandle<AsyncResult<T>>,
	cancelled: Rc<Cell<bool>>,
}

impl<T: 'static> AsyncHandle<T> {
	pub fn spawn<F>(name: CompactString, future: F) -> Self
	where
		F: Future<Output = Option<mlua::Result<T>>> + 'static,
	{
		let cancelled = Rc::new(Cell::new(false));

		// Track cancellation even if the future is dropped before its first poll.
		let guard = guard(cancelled.clone(), |cancelled| {
			cancelled.set(!thread::panicking());
		});

		let handle = task::spawn_local(async move {
			let value = future.await;
			ScopeGuard::into_inner(guard).set(value.is_none());

			AsyncResult { name, value: value.map(|result| result.map_err(Error::from_lua_err)) }
		});

		Self { handle, cancelled }
	}

	pub fn abort(&self) { self.handle.abort(); }

	pub fn cancelled(&self) -> bool { self.cancelled.get() && self.handle.is_finished() }

	pub async fn join(self) -> Result<T, Error> {
		match self.handle.await {
			Ok(mut result) => result.value.take().unwrap_or_else(|| Err(Error::other("Task cancelled"))),
			Err(e) => Err(Error::other(e.to_string())),
		}
	}
}
