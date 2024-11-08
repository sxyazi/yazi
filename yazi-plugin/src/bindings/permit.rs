use std::{mem, ops::Deref};

use mlua::{UserData, prelude::LuaUserDataMethods};
use tokio::sync::SemaphorePermit;

pub type PermitRef<F> = mlua::UserDataRef<Permit<F>>;

pub struct Permit<F: FnOnce()> {
	inner:    Option<SemaphorePermit<'static>>,
	destruct: Option<F>,
}

impl<F: FnOnce()> Deref for Permit<F> {
	type Target = Option<SemaphorePermit<'static>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl<F: FnOnce()> Permit<F> {
	pub fn new(inner: SemaphorePermit<'static>, f: F) -> Self {
		Self { inner: Some(inner), destruct: Some(f) }
	}

	fn dropping(&mut self) {
		if let Some(f) = self.destruct.take() {
			f();
		}
		if let Some(p) = self.inner.take() {
			mem::drop(p);
		}
	}
}

impl<F: FnOnce()> Drop for Permit<F> {
	fn drop(&mut self) { self.dropping(); }
}

impl<F: FnOnce()> UserData for Permit<F> {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut("drop", |_, me, ()| Ok(me.dropping()));
	}
}
