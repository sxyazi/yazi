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
		lua.create_function(|_, (name, args): (mlua::String, Table)| {
			let mut cmd = Cmd::new_or(&name.to_str()?, Layer::Mgr)?;
			cmd.args = Sendable::table_to_args(args)?;
			Ok(emit!(Call(cmd)))
		})
	}

	// TODO: remove
	pub(super) fn app_emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (name, args): (String, Table)| {
			crate::deprecate!(
				lua,
				"`ya.app_emit()` is deprecated, use `ya.emit()` instead, in your {}\n\nSee #2653 for more details: https://github.com/sxyazi/yazi/pull/2653"
			);
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

	// TODO: remove
	pub(super) fn input_emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (name, args): (String, Table)| {
			crate::deprecate!(
				lua,
				"`ya.input_emit()` is deprecated, use `ya.emit()` instead, in your {}\n\nSee #2653 for more details: https://github.com/sxyazi/yazi/pull/2653"
			);
			emit!(Call(Cmd { name, args: Sendable::table_to_args(args)?, layer: Layer::Input }));
			Ok(())
		})
	}
}
