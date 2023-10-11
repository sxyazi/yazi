use core::Ctx;

pub use mlua::Scope;

use crate::{bindings, GLOBALS, LUA};

pub fn scope<'a>(cx: &'a Ctx, f: impl FnOnce(&Scope<'a, 'a>)) {
	let _ = LUA.scope(|scope| {
		let tbl = LUA.create_table()?;
		tbl.set("active", bindings::Active::new(scope, cx).make()?)?;
		tbl.set("tabs", bindings::Tabs::new(scope, cx.manager.tabs()).make()?)?;
		tbl.set("tasks", bindings::Tasks::new(scope, &cx.tasks).make()?)?;
		GLOBALS.set("cx", tbl)?;

		Ok(f(scope))
	});
}
