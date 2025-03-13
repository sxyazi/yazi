use mlua::{AnyUserData, ExternalError, Function, Lua};
use yazi_proxy::{AppProxy, HIDER};

use super::Utils;
use crate::{Id, bindings::{Permit, PermitRef}};

impl Utils {
	pub(super) fn id(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, type_: mlua::String| {
			Ok(Id(match type_.as_bytes().as_ref() {
				b"ft" => yazi_fs::FILES_TICKET.next(),
				_ => Err("Invalid id type".into_lua_err())?,
			}))
		})
	}

	pub(super) fn hide(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, ()| async move {
			if lua.named_registry_value::<PermitRef<fn()>>("HIDE_PERMIT").is_ok_and(|h| h.is_some()) {
				return Err("Cannot hide while already hidden".into_lua_err());
			}

			let permit = HIDER.acquire().await.unwrap();
			AppProxy::stop().await;

			lua.set_named_registry_value("HIDE_PERMIT", Permit::new(permit, AppProxy::resume as fn()))?;
			lua.named_registry_value::<AnyUserData>("HIDE_PERMIT")
		})
	}
}
