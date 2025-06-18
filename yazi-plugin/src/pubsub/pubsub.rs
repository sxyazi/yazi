use mlua::{ExternalResult, Function, Lua, Value};
use yazi_binding::Id;
use yazi_dds::body::Body;

use crate::runtime;

pub struct Pubsub;

impl Pubsub {
	pub(super) fn r#pub(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (kind, value): (mlua::String, Value)| {
			yazi_dds::Pubsub::r#pub(Body::from_lua(&kind.to_str()?, value)?).into_lua_err()
		})
	}

	pub(super) fn pub_to(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (receiver, kind, value): (Id, mlua::String, Value)| {
			yazi_dds::Pubsub::pub_to(*receiver, Body::from_lua(&kind.to_str()?, value)?).into_lua_err()
		})
	}

	pub(super) fn sub(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (kind, f): (mlua::String, Function)| {
			let rt = runtime!(lua)?;
			let Some(cur) = rt.current() else {
				return Err("`sub()` must be called in a sync plugin").into_lua_err();
			};
			if !yazi_dds::Pubsub::sub(cur, &kind.to_str()?, f) {
				return Err("`sub()` called twice").into_lua_err();
			}
			Ok(())
		})
	}

	pub(super) fn sub_remote(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (kind, f): (mlua::String, Function)| {
			let rt = runtime!(lua)?;
			let Some(cur) = rt.current() else {
				return Err("`sub_remote()` must be called in a sync plugin").into_lua_err();
			};
			if !yazi_dds::Pubsub::sub_remote(cur, &kind.to_str()?, f) {
				return Err("`sub_remote()` called twice").into_lua_err();
			}
			Ok(())
		})
	}

	pub(super) fn unsub(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, kind: mlua::String| {
			if let Some(cur) = runtime!(lua)?.current() {
				Ok(yazi_dds::Pubsub::unsub(cur, &kind.to_str()?))
			} else {
				Err("`unsub()` must be called in a sync plugin").into_lua_err()
			}
		})
	}

	pub(super) fn unsub_remote(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, kind: mlua::String| {
			if let Some(cur) = runtime!(lua)?.current() {
				Ok(yazi_dds::Pubsub::unsub_remote(cur, &kind.to_str()?))
			} else {
				Err("`unsub_remote()` must be called in a sync plugin").into_lua_err()
			}
		})
	}
}
