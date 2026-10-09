use std::{future::Future, pin::Pin, task::{Context, Poll}};

use mlua::{Function, Lua, MultiValue, function::AsyncCallFuture};

use super::Runtime;
use crate::runtime_mut;

pub struct RuntimeCo {
	rt:     Runtime,
	lua:    Lua,
	future: Option<AsyncCallFuture<MultiValue>>,
}

impl RuntimeCo {
	pub fn new<R>(rt: R, lua: Lua, f: Function, args: MultiValue) -> Self
	where
		R: Into<Runtime>,
	{
		Self { rt: rt.into(), lua, future: Some(f.call_async(args)) }
	}
}

impl Future for RuntimeCo {
	type Output = mlua::Result<MultiValue>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let me = self.get_mut();
		let lua = &me.lua;

		runtime_mut!(lua)?.swap(&mut me.rt);
		let result = Pin::new(me.future.as_mut().unwrap()).poll(cx);
		runtime_mut!(lua)?.swap(&mut me.rt);
		result
	}
}

impl Drop for RuntimeCo {
	fn drop(&mut self) {
		let lua = &self.lua;

		// Dropping the coroutine runs pending `__close` handlers, which need its runtime context.
		_ = runtime_mut!(lua).map(|mut rt| rt.swap(&mut self.rt));
		drop(self.future.take());
		_ = runtime_mut!(lua).map(|mut rt| rt.swap(&mut self.rt));
	}
}
