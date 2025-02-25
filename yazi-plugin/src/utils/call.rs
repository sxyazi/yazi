use mlua::{Function, Lua, ObjectLike, Table};
use tracing::error;
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_macro::{emit, render};
use yazi_shared::{Layer, event::Cmd};

use super::Utils;

impl Utils {
	pub(super) fn render(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ()| {
			render!();
			Ok(())
		})
	}

	pub(super) fn redraw_with(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, c: Table| {
			let id: mlua::String = c.get("_id")?;

			let mut layout = LAYOUT.get();
			match id.as_bytes().as_ref() {
				b"current" => layout.current = *c.raw_get::<crate::elements::Rect>("_area")?,
				b"preview" => layout.preview = *c.raw_get::<crate::elements::Rect>("_area")?,
				b"progress" => layout.progress = *c.raw_get::<crate::elements::Rect>("_area")?,
				_ => {}
			}

			LAYOUT.set(layout);
			match c.call_method::<Table>("redraw", ()) {
				Err(e) => {
					error!("Failed to `redraw()` the `{}` component:\n{e}", id.display());
					lua.create_table()
				}
				ok => ok,
			}
		})
	}

	pub(super) fn app_emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (name, args): (String, Table)| {
			emit!(Call(Cmd { name, args: Sendable::table_to_args(args)?, layer: Layer::App }));
			Ok(())
		})
	}

	pub(super) fn mgr_emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (name, args): (String, Table)| {
			emit!(Call(Cmd { name, args: Sendable::table_to_args(args)?, layer: Layer::Mgr }));
			Ok(())
		})
	}

	pub(super) fn input_emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (name, args): (String, Table)| {
			emit!(Call(Cmd { name, args: Sendable::table_to_args(args)?, layer: Layer::Input }));
			Ok(())
		})
	}
}
