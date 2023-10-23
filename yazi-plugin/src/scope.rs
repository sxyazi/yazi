pub use mlua::Scope;
use yazi_core::Ctx;

use crate::{bindings, GLOBALS, LUA};

pub fn scope<'a>(cx: &'a Ctx, f: impl FnOnce(&Scope<'a, 'a>)) {
	_ = LUA.scope(|scope| {
		let tbl = LUA.create_table()?;
		tbl.set("active", bindings::Active::new(scope, cx).make()?)?;
		tbl.set("tabs", bindings::Tabs::new(scope, &cx.manager.tabs).make()?)?;
		tbl.set("tasks", bindings::Tasks::new(scope, &cx.tasks).make()?)?;
		GLOBALS.set("cx", tbl)?;

		Ok(f(scope))
	});
}
