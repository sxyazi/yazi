use std::any::TypeId;

use mlua::{AnyUserData, ExternalError, Function, Lua};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout};
use yazi_binding::Id;

use super::Utils;

impl Utils {
	pub(super) fn id(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, r#type: mlua::String| {
			Ok(Id(match &*r#type.as_bytes() {
				b"app" => *yazi_dds::ID,
				b"ft" => yazi_fs::FILES_TICKET.next(),
				_ => Err("Invalid id type".into_lua_err())?,
			}))
		})
	}

	pub(super) fn drop(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ud: AnyUserData| {
			match ud.type_id() {
				Some(t) if t == TypeId::of::<ChildStdin>() => {}
				Some(t) if t == TypeId::of::<ChildStdout>() => {}
				Some(t) if t == TypeId::of::<ChildStderr>() => {}
				Some(t) => Err(format!("Cannot drop userdata of type {t:?}").into_lua_err())?,
				None => Err("Cannot drop scoped userdata".into_lua_err())?,
			};
			ud.destroy()
		})
	}
}
