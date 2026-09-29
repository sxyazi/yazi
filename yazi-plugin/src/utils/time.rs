use std::time::{Instant, SystemTime, UNIX_EPOCH};

use mlua::{ExternalError, FromLua, Function, IntoLuaMulti, Lua, Value};
use yazi_binding::Date;

use super::Utils;

impl Utils {
	pub(super) fn time(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ()| {
			Ok(SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok())
		})
	}

	pub(super) fn date(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, value: Option<Value>| {
			let Some(value) = value else {
				return Date::now().into_lua_multi(lua);
			};

			match Date::from_lua(value, lua) {
				Ok(date) => date.into_lua_multi(lua),
				Err(e) => (Value::Nil, e).into_lua_multi(lua),
			}
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

	pub(super) fn throttle(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (secs, f): (f64, Function)| {
			if secs < 0.0 {
				return Err("negative throttle duration".into_lua_err());
			}

			let mut last = Instant::now();
			lua.create_function_mut(move |_, force: bool| {
				if force || last.elapsed().as_secs_f64() >= secs {
					last = Instant::now();
					f.call::<()>(())?;
				}
				Ok(())
			})
		})
	}
}
