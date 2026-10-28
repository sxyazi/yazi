use mlua::{ExternalError, Function, IntoLua, Lua, Table};
use tokio::sync::mpsc;
use yazi_binding::runtime;
use yazi_macro::emit;
use yazi_shared::{Layer, Source, data::DataKey, event::{Action, Event}, scope::Scope, sendable::Sendable};

use super::Utils;

impl Utils {
	pub(super) fn emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (name, args): (String, Table)| {
			let mut action = Action::new(name, Source::Emit, Layer::Mgr)?;
			action.args = Sendable::table_to_args(lua, args)?;
			// TODO(scope): make a new Data::Scope variant to avoid the conversion to Any2 and back
			action.scope = action
				.args
				.remove(&DataKey::from("scope"))
				.map(Scope::try_from)
				.transpose()?
				.unwrap_or_default();

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
			// TODO(scope): make a new Data::Scope variant to avoid the conversion to Any2 and back
			action.scope = action
				.args
				.remove(&DataKey::from("scope"))
				.map(Scope::try_from)
				.transpose()?
				.unwrap_or_default();

			let (tx, mut rx) = mpsc::unbounded_channel();
			emit!(Call(action.with_replier(tx)));

			rx.recv()
				.await
				.ok_or_else(|| "channel closed before action response".into_lua_err())??
				.into_lua(&lua)
		})
	}
}
