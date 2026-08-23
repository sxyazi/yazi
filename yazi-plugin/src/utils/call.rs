use mlua::{ExternalError, Function, Lua, Table};
use tokio::sync::mpsc;
use yazi_binding::runtime;
use yazi_macro::emit;
use yazi_shared::{Layer, Source, data::Sendable, event::{Action, Event}};

use super::Utils;

impl Utils {
	pub(super) fn emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (name, args): (String, Table)| {
			let mut action = Action::new(name, Source::Emit, Layer::Mgr)?;
			action.args = Sendable::table_to_args(lua, args)?;

			let event = Event::Call(action.into());
			if runtime!(lua)?.is_blocking() {
				event.preempt();
			} else {
				event.emit();
			}

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
