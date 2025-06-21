use std::collections::HashMap;

use futures::future::BoxFuture;
use yazi_shared::Id;

#[derive(Default)]
pub(super) struct Hooks {
	inner: HashMap<Id, Hook>,
}

impl Hooks {
	#[allow(dead_code)]
	pub(super) fn add_sync<F>(&mut self, id: Id, f: F)
	where
		F: FnOnce(bool) + Send + Sync + 'static,
	{
		self.inner.insert(id, Hook::Sync(Box::new(f)));
	}

	pub(super) fn add_async<F>(&mut self, id: Id, f: F)
	where
		F: FnOnce(bool) -> BoxFuture<'static, ()> + Send + Sync + 'static,
	{
		self.inner.insert(id, Hook::Async(Box::new(f)));
	}

	pub(super) fn run_or_pop(&mut self, id: Id, cancel: bool) -> Option<BoxFuture<'static, ()>> {
		match self.inner.remove(&id)? {
			Hook::Sync(f) => {
				f(cancel);
				None
			}
			Hook::Async(f) => Some(f(cancel)),
		}
	}
}

// --- Hook
pub(super) enum Hook {
	Sync(Box<dyn FnOnce(bool) + Send>),
	Async(Box<dyn (FnOnce(bool) -> BoxFuture<'static, ()>) + Send>),
}
