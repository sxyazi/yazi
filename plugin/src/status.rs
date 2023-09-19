use core::Ctx;

use mlua::{Function, Result};

use crate::{bindings, LUA};

pub struct Status;

impl Status {
	fn scoped<T, F: FnOnce() -> Result<T>>(cx: &Ctx, f: F) -> Result<T> {
		LUA.scope(|scope| {
			let manager = scope.create_nonstatic_userdata(bindings::Manager::new(&cx.manager))?;
			let tasks = scope.create_nonstatic_userdata(bindings::Tasks::new(&cx.tasks))?;

			let cx = LUA.create_table()?;
			cx.set("manager", manager)?;
			cx.set("tasks", tasks)?;
			LUA.globals().set("cx", cx)?;

			f()
		})
	}

	pub fn mode(cx: &Ctx) -> Result<String> {
		Self::scoped(cx, || {
			let mode: Function = LUA.globals().get("mode")?;
			mode.call::<_, String>(())
		})
	}

	pub fn size(cx: &Ctx) -> Result<String> {
		Self::scoped(cx, || {
			let size: Function = LUA.globals().get("size")?;
			size.call::<_, String>(())
		})
	}

	pub fn name(cx: &Ctx) -> Result<String> {
		Self::scoped(cx, || {
			let size: Function = LUA.globals().get("name")?;
			size.call::<_, String>(())
		})
	}

	pub fn permissions(cx: &Ctx) -> Result<String> {
		Self::scoped(cx, || {
			let size: Function = LUA.globals().get("permissions")?;
			size.call::<_, String>(())
		})
	}

	pub fn percentage(cx: &Ctx) -> Result<String> {
		Self::scoped(cx, || {
			let size: Function = LUA.globals().get("percentage")?;
			size.call::<_, String>(())
		})
	}

	pub fn position(cx: &Ctx) -> Result<String> {
		Self::scoped(cx, || {
			let size: Function = LUA.globals().get("position")?;
			size.call::<_, String>(())
		})
	}
}
