use mlua::{UserData, UserDataMethods};
use tokio::task::JoinHandle;
use yazi_binding::Scope;

pub struct EvaluateHandle {
	scope:  Scope,
	handle: JoinHandle<()>,
}

impl EvaluateHandle {
	pub(super) fn new(scope: Scope, handle: JoinHandle<()>) -> Self { Self { scope, handle } }

	fn abort(&self) {
		self.scope.cancel();
		self.handle.abort();
	}
}

impl Drop for EvaluateHandle {
	fn drop(&mut self) { self.abort(); }
}

impl UserData for EvaluateHandle {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("abort", |_, me, ()| Ok(me.abort()));
		methods.add_async_method_once("wait", |_, mut me, ()| async move {
			(&mut me.handle).await.ok();
			Ok(())
		});
	}
}
