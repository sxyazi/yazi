use std::sync::Arc;

use mlua::{Lua, Table};
use users::{Groups, Users, UsersCache};

use super::Utils;

impl Utils {
	pub(super) fn unix_user(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		let arc_cache = Arc::new(UsersCache::new());

		{
			let cache = Arc::clone(&arc_cache);
			ya.set("get_current_uid", lua.create_function(move |_, ()| Ok(cache.get_current_uid()))?)?;
		}

		{
			let cache = Arc::clone(&arc_cache);
			ya.set("get_current_gid", lua.create_function(move |_, ()| Ok(cache.get_current_gid()))?)?;
		}

		{
			let cache = Arc::clone(&arc_cache);
			ya.set(
				"get_user_name_by_uid",
				lua.create_function(move |_, uid: u32| {
					Ok(
						cache
							.get_user_by_uid(uid)
							.and_then(|u| u.name().to_str().and_then(|n| Some(n.to_owned()))),
					)
				})?,
			)?;
		}

		{
			let cache = Arc::clone(&arc_cache);
			ya.set(
				"get_group_name_by_gid",
				lua.create_function(move |_, gid: u32| {
					Ok(
						cache
							.get_group_by_gid(gid)
							.and_then(|g| g.name().to_str().and_then(|n| Some(n.to_owned()))),
					)
				})?,
			)?;
		}

		Ok(())
	}
}
