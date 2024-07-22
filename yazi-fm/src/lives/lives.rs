use std::mem;

use mlua::Scope;
use scopeguard::defer;
use tracing::error;
use yazi_plugin::LUA;
use yazi_shared::RoCell;

use crate::Ctx;

pub(super) static SCOPE: RoCell<&mlua::Scope> = RoCell::new();

pub(crate) struct Lives;

impl Lives {
	pub(crate) fn register() -> mlua::Result<()> {
		super::Config::register(&LUA)?;
		super::File::register(&LUA)?;
		super::Files::register(&LUA)?;
		super::Filter::register(&LUA)?;
		super::Finder::register(&LUA)?;
		super::Folder::register(&LUA)?;
		super::Mode::register(&LUA)?;
		super::Preview::register(&LUA)?;
		super::Selected::register(&LUA)?;
		super::Tab::register(&LUA)?;
		super::Tabs::register(&LUA)?;
		super::Tasks::register(&LUA)?;
		super::Yanked::register(&LUA)?;

		Ok(())
	}

	pub(crate) fn scope<'a, T>(
		cx: &'a Ctx,
		f: impl FnOnce(&Scope<'a, 'a>) -> mlua::Result<T>,
	) -> mlua::Result<T> {
		let result = LUA.scope(|scope| {
			defer! { SCOPE.drop(); }
			SCOPE.init(unsafe {
				mem::transmute::<&mlua::Scope<'a, 'a>, &mlua::Scope<'static, 'static>>(scope)
			});
			LUA.set_named_registry_value("cx", scope.create_any_userdata_ref(cx)?)?;

			let globals = LUA.globals();
			globals.raw_set(
				"cx",
				LUA.create_table_from([
					("active", super::Tab::make(cx.manager.active())?),
					("tabs", super::Tabs::make(&cx.manager.tabs)?),
					("tasks", super::Tasks::make(&cx.tasks)?),
					("yanked", super::Yanked::make(&cx.manager.yanked)?),
				])?,
			)?;

			f(scope)
		});

		if let Err(ref e) = result {
			error!("{e}");
		}
		result
	}
}
