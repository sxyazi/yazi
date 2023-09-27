use core::Ctx;

use mlua::{Result, Table, TableExt};
use ratatui::layout;

use crate::{bindings, Rect, GLOBALS, LUA};

pub struct Status;

impl Status {
	fn scope<T, F: FnOnce() -> Result<T>>(cx: &Ctx, f: F) -> Result<T> {
		// crate::Manager::register()?;

		LUA.scope(|scope| {
			let manager = crate::Manager::make(scope, &cx.manager)?;
			let tasks = scope.create_nonstatic_userdata(bindings::Tasks::new(&cx.tasks))?;

			let cx3 = LUA.create_table()?;
			cx3.set("manager", manager)?;
			cx3.set("tasks", tasks)?;

			GLOBALS.set("cx", cx3)?;

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
