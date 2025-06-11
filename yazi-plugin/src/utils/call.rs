use mlua::{Function, Lua, Table};
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

	pub(super) fn emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (name, args): (String, Table)| {
			let mut cmd = Cmd::new_or(name, Layer::Mgr)?;
			cmd.args = Sendable::table_to_args(args)?;
			Ok(emit!(Call(cmd)))
		})
	}

	pub(super) fn mgr_emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (name, args): (String, Table)| {
			emit!(Call(Cmd {
				name:  name.into(),
				args:  Sendable::table_to_args(args)?,
				layer: Layer::Mgr,
			}));
			Ok(())
		})
	}
}
