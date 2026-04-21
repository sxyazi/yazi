use mlua::{ExternalError, Function, Lua, Table};
use tokio::sync::mpsc;
use yazi_binding::deprecate;
use yazi_dds::Sendable;
use yazi_macro::emit;
use yazi_shared::{Layer, Source, event::{Action, Cmd}};

use super::Utils;

impl Utils {
	pub(super) fn emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (name, args): (String, Table)| {
			let mut action = Action::new(name, Source::Emit, Layer::Mgr)?;
			action.args = Sendable::table_to_args(lua, args)?;
			Ok(emit!(Call(action)))
		})
	}

	pub(super) fn mgr_emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (name, args): (String, Table)| {
			deprecate!(lua, "ya.mgr_emit() has been deprecated since v25.5.28 and will soon-to-be removed in a future release. \n\nUse ya.emit() in your {} instead, see #2653 for details: https://github.com/sxyazi/yazi/pull/2653");
			emit!(Call(Action {
				cmd: Cmd { name: name.into(), args: Sendable::table_to_args(lua, args)? },
				layer:  Layer::Mgr,
				source: Source::Emit,
			}));
			Ok(())
		})
	}

	pub(super) fn exec(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, (name, args): (String, Table)| async move {
			let mut action = Action::new(name, Source::Emit, Layer::Mgr)?;
			action.args = Sendable::table_to_args(&lua, args)?;

			let (tx, mut rx) = mpsc::unbounded_channel();
			emit!(Call(action.with_replier(tx)));

			Sendable::data_to_value(
				&lua,
				rx.recv().await.ok_or_else(|| "channel closed before action response".into_lua_err())??,
			)
		})
	}
}
