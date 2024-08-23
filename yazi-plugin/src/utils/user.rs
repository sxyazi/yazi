use mlua::{Lua, Table};

use super::Utils;

impl Utils {
	#[cfg(unix)]
	pub(super) fn user(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		use uzers::{Groups, Users};
		use yazi_shared::{hostname, USERS_CACHE};

		use crate::utils::HOSTNAME_CACHE;

		ya.raw_set("uid", lua.create_function(|_, ()| Ok(USERS_CACHE.get_current_uid()))?)?;

		ya.raw_set("gid", lua.create_function(|_, ()| Ok(USERS_CACHE.get_current_gid()))?)?;

		ya.raw_set(
			"user_name",
			lua.create_function(|lua, uid: Option<u32>| {
				USERS_CACHE
					.get_user_by_uid(uid.unwrap_or_else(|| USERS_CACHE.get_current_uid()))
					.map(|s| lua.create_string(s.name().as_encoded_bytes()))
					.transpose()
			})?,
		)?;

		ya.raw_set(
			"group_name",
			lua.create_function(|lua, gid: Option<u32>| {
				USERS_CACHE
					.get_group_by_gid(gid.unwrap_or_else(|| USERS_CACHE.get_current_gid()))
					.map(|s| lua.create_string(s.name().as_encoded_bytes()))
					.transpose()
			})?,
		)?;

		ya.raw_set(
			"host_name",
			lua.create_function(|lua, ()| {
				HOSTNAME_CACHE
					.get_or_init(|| hostname().ok())
					.as_ref()
					.map(|s| lua.create_string(s))
					.transpose()
			})?,
		)?;

		Ok(())
	}

	#[cfg(windows)]
	pub(super) fn user(_lua: &Lua, _ya: &Table) -> mlua::Result<()> { Ok(()) }
}
