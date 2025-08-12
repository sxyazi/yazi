use std::collections::HashMap;

use futures::future::BoxFuture;
use yazi_shared::Id;

#[derive(Default)]
pub(super) struct Hooks {
	inner: HashMap<Id, Box<dyn Hook>>,
}

impl Hooks {
	#[inline]
	pub(super) fn add_async<F, Fut>(&mut self, id: Id, f: F)
	where
		F: FnOnce(bool) -> Fut + Send + 'static,
		Fut: Future<Output = ()> + Send + 'static,
	{
		self.inner.insert(id, Box::new(f));
	}

	#[inline]
	pub(super) fn pop(&mut self, id: Id) -> Option<Box<dyn Hook>> { self.inner.remove(&id) }
}

// --- Hook
pub trait Hook: Send {
	fn call(self: Box<Self>, cancel: bool) -> BoxFuture<'static, ()>;
}

impl<F, Fut> Hook for F
where
	F: FnOnce(bool) -> Fut + Send,
	Fut: Future<Output = ()> + Send + 'static,
{
	fn call(self: Box<Self>, cancel: bool) -> BoxFuture<'static, ()> { Box::pin((*self)(cancel)) }
}
