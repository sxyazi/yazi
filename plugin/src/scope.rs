use core::Ctx;

use crate::{bindings, GLOBALS, LUA};

pub fn scope<F: FnOnce()>(cx: &Ctx, f: F) {
	let _ = LUA.scope(|scope| {
		let tbl = LUA.create_table()?;
		tbl.set("active", bindings::Tab::new(scope, cx, cx.manager.active()).make()?)?;
		tbl.set("tasks", bindings::Tasks::make(scope, &cx.tasks)?)?;
		GLOBALS.set("cx", tbl)?;

		Ok(f())
	});
}
