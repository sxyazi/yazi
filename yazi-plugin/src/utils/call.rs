use mlua::{ExternalError, Function, Lua, Table};
use tokio::sync::mpsc;
use yazi_dds::Sendable;
use yazi_macro::emit;
use yazi_shared::{Layer, Source, event::Action};

use super::Utils;

impl Utils {
	pub(super) fn emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (name, args): (String, Table)| {
			let mut action = Action::new(name, Source::Emit, Some(Layer::Mgr))?;
			action.args = Sendable::table_to_args(lua, args)?;
			Ok(emit!(Call(action)))
		})
	}

	pub(super) fn exec(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, (name, args): (String, Table)| async move {
			let mut action = Action::new(name, Source::Emit, Some(Layer::Mgr))?;
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
