use core::Ctx;

use mlua::{Result, Table, TableExt};
use ratatui::layout;

use crate::{bindings, Rect, GLOBALS, LUA};

pub struct Status;

impl Status {
	fn scope<T, F: FnOnce() -> Result<T>>(cx: &Ctx, f: F) -> Result<T> {
		// crate::Manager::register()?;

		LUA.scope(|scope| {
			let manager = bindings::Manager::make(scope, &cx.manager)?;
			let tasks = bindings::Tasks::make(scope, &cx.tasks)?;

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
