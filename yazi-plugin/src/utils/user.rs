use mlua::{Lua, Table};
use uzers::{Groups, Users, UsersCache};
use yazi_shared::RoCell;

use super::Utils;

static CACHE: RoCell<UsersCache> = RoCell::new();

impl Utils {
	pub(super) fn user(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		CACHE.with(Default::default);

		ya.set("uid", lua.create_function(|_, ()| Ok(CACHE.get_current_uid()))?)?;

		ya.set("gid", lua.create_function(|_, ()| Ok(CACHE.get_current_gid()))?)?;

		ya.set(
			"user_name",
			lua.create_function(|lua, uid: Option<u32>| {
				CACHE
					.get_user_by_uid(uid.unwrap_or_else(|| CACHE.get_current_uid()))
					.map(|s| lua.create_string(s.name().as_encoded_bytes()))
					.transpose()
			})?,
		)?;

		ya.set(
			"group_name",
			lua.create_function(|lua, gid: Option<u32>| {
				CACHE
					.get_group_by_gid(gid.unwrap_or_else(|| CACHE.get_current_gid()))
					.map(|s| lua.create_string(s.name().as_encoded_bytes()))
					.transpose()
			})?,
		)?;

		Ok(())
	}
}
