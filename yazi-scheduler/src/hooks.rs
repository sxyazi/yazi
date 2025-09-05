use futures::{FutureExt, future::BoxFuture};
use hashbrown::HashMap;
use yazi_shared::Id;

#[derive(Default)]
pub(super) struct Hooks {
	inner: HashMap<Id, Box<dyn Hook>>,
}

impl Hooks {
	#[inline]
	pub(super) fn add_sync<F>(&mut self, id: Id, f: F)
	where
		F: FnOnce(bool) + Send + 'static,
	{
		self.inner.insert(id, Box::new(SyncHook(f)));
	}

	#[inline]
	pub(super) fn add_async<F, Fut>(&mut self, id: Id, f: F)
	where
		F: FnOnce(bool) -> Fut + Send + 'static,
		Fut: Future<Output = ()> + Send + 'static,
	{
		self.inner.insert(id, Box::new(AsyncHook(f)));
	}

	#[inline]
	pub(super) fn pop(&mut self, id: Id) -> Option<Box<dyn Hook>> { self.inner.remove(&id) }
}

// --- Hook
pub(super) trait Hook: Send {
	fn call(self: Box<Self>, cancel: bool) -> Option<BoxFuture<'static, ()>>;
}

struct SyncHook<F>(F);

impl<F> Hook for SyncHook<F>
where
	F: FnOnce(bool) + Send,
{
	fn call(self: Box<Self>, cancel: bool) -> Option<BoxFuture<'static, ()>> {
		(self.0)(cancel);
		None
	}
}

struct AsyncHook<F>(F);

impl<F, Fut> Hook for AsyncHook<F>
where
	F: FnOnce(bool) -> Fut + Send,
	Fut: Future<Output = ()> + Send + 'static,
{
	fn call(self: Box<Self>, cancel: bool) -> Option<BoxFuture<'static, ()>> {
		Some((self.0)(cancel).boxed())
	}
}
