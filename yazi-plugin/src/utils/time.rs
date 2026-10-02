use std::time::{Duration, Instant};

use mlua::{ExternalResult, FromLua, Function, IntoLuaMulti, Lua, Value};
use yazi_binding::time::Time;

use super::Utils;

impl Utils {
	pub(super) fn time(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, value: Option<Value>| {
			match value.map(|value| Time::from_lua(value, lua)).transpose() {
				Ok(time) => time.unwrap_or_else(Time::now).into_lua_multi(lua),
				Err(e) => (Value::Nil, e).into_lua_multi(lua),
			}
		})
	}

	pub(super) fn sleep(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|_, secs: f64| async move {
			tokio::time::sleep(Duration::try_from_secs_f64(secs).into_lua_err()?).await;
			Ok(())
		})
	}

	pub(super) fn throttle(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (secs, f): (f64, Function)| {
			let dur = Duration::try_from_secs_f64(secs).into_lua_err()?;
			let mut last = Instant::now();
			lua.create_function_mut(move |_, force: bool| {
				if force || last.elapsed() >= dur {
					last = Instant::now();
					f.call::<()>(())?;
				}
				Ok(())
			})
		})
	}
}
