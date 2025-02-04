#[cfg(unix)]
use mlua::{Function, Lua};

use super::Utils;

impl Utils {
	#[cfg(unix)]
	pub(super) fn uid(lua: &Lua) -> mlua::Result<Function> {
		use uzers::Users;
		lua.create_function(|_, ()| Ok(yazi_shared::USERS_CACHE.get_current_uid()))
	}

	#[cfg(unix)]
	pub(super) fn gid(lua: &Lua) -> mlua::Result<Function> {
		use uzers::Groups;
		lua.create_function(|_, ()| Ok(yazi_shared::USERS_CACHE.get_current_gid()))
	}

	#[cfg(unix)]
	pub(super) fn user_name(lua: &Lua) -> mlua::Result<Function> {
		use uzers::Users;
		use yazi_shared::USERS_CACHE;

		lua.create_function(|lua, uid: Option<u32>| {
			USERS_CACHE
				.get_user_by_uid(uid.unwrap_or_else(|| USERS_CACHE.get_current_uid()))
				.map(|s| lua.create_string(s.name().as_encoded_bytes()))
				.transpose()
		})
	}

	#[cfg(unix)]
	pub(super) fn group_name(lua: &Lua) -> mlua::Result<Function> {
		use uzers::Groups;
		use yazi_shared::USERS_CACHE;

		lua.create_function(|lua, gid: Option<u32>| {
			USERS_CACHE
				.get_group_by_gid(gid.unwrap_or_else(|| USERS_CACHE.get_current_gid()))
				.map(|s| lua.create_string(s.name().as_encoded_bytes()))
				.transpose()
		})
	}

	#[cfg(unix)]
	pub(super) fn host_name(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, ()| yazi_shared::hostname().map(|s| lua.create_string(s)).transpose())
	}
}
