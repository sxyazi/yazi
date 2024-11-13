use std::time::{SystemTime, UNIX_EPOCH};

use mlua::{ExternalError, Function, Lua};

use super::Utils;

impl Utils {
	pub(super) fn time(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ()| {
			Ok(SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok())
		})
	}

	pub(super) fn sleep(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|_, secs: f64| async move {
			if secs < 0.0 {
				return Err("negative sleep duration".into_lua_err());
			}

			tokio::time::sleep(tokio::time::Duration::from_secs_f64(secs)).await;
			Ok(())
		})
	}
}
