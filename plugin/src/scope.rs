use core::Ctx;

pub use mlua::Scope;

use crate::{bindings, GLOBALS, LUA};

pub fn scope<'a>(cx: &'a Ctx, f: impl FnOnce(&Scope<'a, 'a>)) {
	let _ = LUA.scope(|scope| {
		let tbl = LUA.create_table()?;
		tbl.set("active", bindings::Tab::new(scope, cx, cx.manager.active()).make()?)?;
		tbl.set("tasks", bindings::Tasks::make(scope, &cx.tasks)?)?;
		GLOBALS.set("cx", tbl)?;

		Ok(f(scope))
	});
}
