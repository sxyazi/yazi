use std::any::TypeId;

use mlua::{AnyUserData, ExternalError, Function, IntoLua, Lua, LuaString};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout};
use yazi_shared::{path::PathBufDyn, url::UrlBuf};
use yazi_vfs::engine::RwFile;

use super::Utils;

impl Utils {
	pub(super) fn clone(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, ud: AnyUserData| match ud.type_id() {
			Some(t) if t == TypeId::of::<UrlBuf>() => ud.borrow::<UrlBuf>()?.clone().into_lua(lua),
			Some(t) if t == TypeId::of::<PathBufDyn>() => {
				ud.borrow::<PathBufDyn>()?.clone().into_lua(lua)
			}
			Some(t) => Err(format!("Cannot clone userdata of type {t:?}").into_lua_err()),
			None => Err("Cannot clone scoped userdata".into_lua_err()),
		})
	}

	pub(super) fn drop(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ud: AnyUserData| {
			match ud.type_id() {
				Some(t) if t == TypeId::of::<RwFile>() => {}
				Some(t) if t == TypeId::of::<ChildStdin>() => {}
				Some(t) if t == TypeId::of::<ChildStdout>() => {}
				Some(t) if t == TypeId::of::<ChildStderr>() => {}
				Some(t) => Err(format!("Cannot drop userdata of type {t:?}").into_lua_err())?,
				None => Err("Cannot drop scoped userdata".into_lua_err())?,
			};
			ud.destroy()
		})
	}

	pub(super) fn id(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, r#type: LuaString| {
			Ok(match &*r#type.as_bytes() {
				b"app" => *yazi_boot::ID,
				b"ft" => yazi_fs::FILES_TICKET.next(),
				_ => Err("Invalid id type".into_lua_err())?,
			})
		})
	}
}
