use mlua::{ExternalResult, Function, Lua, Value};
use yazi_binding::{Id, runtime};
use yazi_dds::ember::Ember;

pub struct Pubsub;

impl Pubsub {
	pub(super) fn r#pub(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (kind, value): (mlua::String, Value)| {
			yazi_dds::Pubsub::r#pub(Ember::from_lua(lua, &kind.to_str()?, value)?).into_lua_err()
		})
	}

	pub(super) fn pub_to(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (receiver, kind, value): (Id, mlua::String, Value)| {
			yazi_dds::Pubsub::pub_to(*receiver, Ember::from_lua(lua, &kind.to_str()?, value)?)
				.into_lua_err()
		})
	}

	pub(super) fn sub(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (kind, f): (mlua::String, Function)| {
			let rt = runtime!(lua)?;
			if !yazi_dds::Pubsub::sub(rt.current()?, &kind.to_str()?, f) {
				return Err("`sub()` called twice").into_lua_err();
			}
			Ok(())
		})
	}

	pub(super) fn sub_remote(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (kind, f): (mlua::String, Function)| {
			let rt = runtime!(lua)?;
			if !yazi_dds::Pubsub::sub_remote(rt.current()?, &kind.to_str()?, f) {
				return Err("`sub_remote()` called twice").into_lua_err();
			}
			Ok(())
		})
	}

	pub(super) fn unsub(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, kind: mlua::String| {
			let rt = runtime!(lua)?;
			Ok(yazi_dds::Pubsub::unsub(rt.current()?, &kind.to_str()?))
		})
	}

	pub(super) fn unsub_remote(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, kind: mlua::String| {
			let rt = runtime!(lua)?;
			Ok(yazi_dds::Pubsub::unsub_remote(rt.current()?, &kind.to_str()?))
		})
	}
}
