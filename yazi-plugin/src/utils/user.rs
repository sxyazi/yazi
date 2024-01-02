use mlua::{Lua, Table};

use super::Utils;

impl Utils {
	#[cfg(unix)]
	pub(super) fn user(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		use uzers::{Groups, Users};

		use crate::utils::USERS_CACHE;

		ya.set("uid", lua.create_function(|_, ()| Ok(USERS_CACHE.get_current_uid()))?)?;

		ya.set("gid", lua.create_function(|_, ()| Ok(USERS_CACHE.get_current_gid()))?)?;

		ya.set(
			"user_name",
			lua.create_function(|lua, uid: Option<u32>| {
				USERS_CACHE
					.get_user_by_uid(uid.unwrap_or_else(|| USERS_CACHE.get_current_uid()))
					.map(|s| lua.create_string(s.name().as_encoded_bytes()))
					.transpose()
			})?,
		)?;

		ya.set(
			"group_name",
			lua.create_function(|lua, gid: Option<u32>| {
				USERS_CACHE
					.get_group_by_gid(gid.unwrap_or_else(|| USERS_CACHE.get_current_gid()))
					.map(|s| lua.create_string(s.name().as_encoded_bytes()))
					.transpose()
			})?,
		)?;

		Ok(())
	}

	#[cfg(windows)]
	pub(super) fn user(_lua: &Lua, _ya: &Table) -> mlua::Result<()> { Ok(()) }
}
