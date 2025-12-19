use std::{mem, ops::Deref};

use futures::{FutureExt, future::BoxFuture};
use mlua::{UserData, prelude::LuaUserDataMethods};
use tokio::sync::SemaphorePermit;

pub type PermitRef = mlua::UserDataRef<Permit>;

pub struct Permit {
	inner:    Option<SemaphorePermit<'static>>,
	destruct: Option<BoxFuture<'static, ()>>,
}

impl Deref for Permit {
	type Target = Option<SemaphorePermit<'static>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Permit {
	pub fn new<F>(inner: SemaphorePermit<'static>, f: F) -> Self
	where
		F: Future<Output = ()> + 'static + Send,
	{
		Self { inner: Some(inner), destruct: Some(f.boxed()) }
	}

	fn dropping(&mut self) -> impl Future<Output = ()> + 'static {
		let inner = self.inner.take();
		let destruct = self.destruct.take();

		async move {
			if let Some(f) = destruct {
				f.await;
			}
			if let Some(p) = inner {
				mem::drop(p);
			}
		}
	}
}

impl Drop for Permit {
	fn drop(&mut self) { tokio::spawn(self.dropping()); }
}

impl UserData for Permit {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("drop", |_, mut me, ()| async move { Ok(me.dropping().await) });
	}
}
