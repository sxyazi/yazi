use mlua::{ExternalResult, Function, Lua, Value};
use yazi_dds::body::Body;

use crate::RUNNING;

pub struct Pubsub;

impl Pubsub {
	pub fn install(lua: &'static Lua) -> mlua::Result<()> {
		let ps = lua.create_table()?;

		ps.raw_set(
			"pub",
			lua.create_function(|_, (kind, value): (mlua::String, Value)| {
				yazi_dds::Pubsub::pub_(Body::from_lua(kind.to_str()?, value).into_lua_err()?);
				Ok(())
			})?,
		)?;

		ps.raw_set(
			"pub_to",
			lua.create_function(|_, (receiver, kind, value): (u64, mlua::String, Value)| {
				yazi_dds::Pubsub::pub_to(receiver, Body::from_lua(kind.to_str()?, value).into_lua_err()?);
				Ok(())
			})?,
		)?;

		ps.raw_set(
			"pub_static",
			lua.create_function(|_, (severity, kind, value): (u8, mlua::String, Value)| {
				if severity < 1 {
					return Err("Severity must be at least 1").into_lua_err();
				}

				yazi_dds::Pubsub::pub_static(
					severity,
					Body::from_lua(kind.to_str()?, value).into_lua_err()?,
				);
				Ok(())
			})?,
		)?;

		ps.raw_set(
			"sub",
			lua.create_function(|_, (kind, f): (mlua::String, Function)| {
				let Some(name) = &*RUNNING.load() else {
					return Err("`sub()` must be called in a sync plugin").into_lua_err();
				};
				if !yazi_dds::Pubsub::sub(name, kind.to_str()?, f) {
					return Err("`sub()` called twice").into_lua_err();
				}
				Ok(())
			})?,
		)?;

		ps.raw_set(
			"sub_remote",
			lua.create_function(|_, (kind, f): (mlua::String, Function)| {
				let Some(name) = &*RUNNING.load() else {
					return Err("`sub_remote()` must be called in a sync plugin").into_lua_err();
				};
				if !yazi_dds::Pubsub::sub_remote(name, kind.to_str()?, f) {
					return Err("`sub_remote()` called twice").into_lua_err();
				}
				Ok(())
			})?,
		)?;

		ps.raw_set(
			"unsub",
			lua.create_function(|_, kind: mlua::String| {
				if let Some(name) = &*RUNNING.load() {
					Ok(yazi_dds::Pubsub::unsub(name, kind.to_str()?))
				} else {
					Err("`unsub()` must be called in a sync plugin").into_lua_err()
				}
			})?,
		)?;

		ps.raw_set(
			"unsub_remote",
			lua.create_function(|_, kind: mlua::String| {
				if let Some(name) = &*RUNNING.load() {
					Ok(yazi_dds::Pubsub::unsub_remote(name, kind.to_str()?))
				} else {
					Err("`unsub_remote()` must be called in a sync plugin").into_lua_err()
				}
			})?,
		)?;

		lua.globals().raw_set("ps", ps)
	}
}
