use std::sync::Arc;

use mlua::{Scope, Table};
use tracing::error;
use yazi_config::LAYOUT;
use yazi_plugin::{elements::RectRef, LUA};

use crate::Ctx;

pub(crate) struct Lives;

impl Lives {
	pub(crate) fn register() -> mlua::Result<()> {
		yazi_plugin::bindings::Cha::register(&LUA)?;
		yazi_plugin::bindings::Icon::register(&LUA)?;
		yazi_plugin::bindings::Url::register(&LUA)?;

		super::Active::register(&LUA)?;
		super::Folder::register(&LUA)?;
		super::Tabs::register(&LUA)?;
		super::Tasks::register(&LUA)?;

		Ok(())
	}

	pub(crate) fn scope<'a, T>(
		cx: &'a Ctx,
		f: impl FnOnce(&Scope<'a, 'a>) -> mlua::Result<T>,
	) -> mlua::Result<T> {
		let result = LUA.scope(|scope| {
			LUA.set_named_registry_value("cx", scope.create_any_userdata_ref(cx)?)?;

			let global = LUA.globals();
			global.set(
				"cx",
				LUA.create_table_from([
					("active", super::Active::new(scope, cx.manager.active()).make()?),
					("tabs", super::Tabs::new(scope, &cx.manager.tabs).make()?),
					("tasks", super::Tasks::new(scope, &cx.tasks).make()?),
				])?,
			)?;

			let ret = f(scope)?;

			LAYOUT.store(Arc::new(yazi_config::Layout {
				header:  *global.get::<_, Table>("Header")?.get::<_, RectRef>("area")?,
				parent:  *global.get::<_, Table>("Parent")?.get::<_, RectRef>("area")?,
				current: *global.get::<_, Table>("Current")?.get::<_, RectRef>("area")?,
				preview: *global.get::<_, Table>("Preview")?.get::<_, RectRef>("area")?,
				status:  *global.get::<_, Table>("Status")?.get::<_, RectRef>("area")?,
			}));

			Ok(ret)
		});

		if let Err(ref e) = result {
			error!("{e}");
		}
		result
	}

	pub(crate) fn partial_scope<'a>(cx: &'a Ctx, f: impl FnOnce(&Scope<'a, 'a>)) {
		let result = LUA.scope(|scope| {
			LUA.globals().set(
				"cx",
				LUA.create_table_from([("tasks", super::Tasks::new(scope, &cx.tasks).make()?)])?,
			)?;

			f(scope);
			Ok(())
		});

		if let Err(e) = result {
			error!("{e}");
		}
	}
}
