use core::Ctx;

use mlua::{Function, Result};
use ratatui::layout;

use crate::{bindings, Rect, LUA};

pub struct Status;

impl Status {
	fn scope<T, F: FnOnce() -> Result<T>>(cx: &Ctx, f: F) -> Result<T> {
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

	pub fn layout(cx: &Ctx, area: layout::Rect) -> Result<String> {
		Self::scope(cx, || {
			let layout: Function = LUA.globals().get("layout")?;
			layout.call::<_, String>(Rect::from(area))
		})
	}
}
