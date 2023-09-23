use core::Ctx;

use mlua::{Result, Table, TableExt};
use ratatui::layout;

use crate::{bindings, Rect, GLOBALS, LUA};

pub struct Status;

impl Status {
	fn scope<T, F: FnOnce() -> Result<T>>(cx: &Ctx, f: F) -> Result<T> {
		LUA.scope(|scope| {
			let manager = scope.create_nonstatic_userdata(bindings::Manager::new(&cx.manager))?;
			let tasks = scope.create_nonstatic_userdata(bindings::Tasks::new(&cx.tasks))?;

			let cx = LUA.create_table()?;
			cx.set("manager", manager)?;
			cx.set("tasks", tasks)?;
			GLOBALS.set("cx", cx)?;

			f()
		})
	}

	pub fn render(cx: &Ctx, area: layout::Rect) -> Result<String> {
		Self::scope(cx, || {
			let status: Table = GLOBALS.get("Status")?;
			status.call_method::<_, String>("render", Rect::from(area))
		})
	}
}
