use mlua::{ExternalResult, Function, Lua, Value};
use yazi_dds::body::Body;

use crate::runtime::RtRef;

pub struct Pubsub;

impl Pubsub {
	pub(super) fn install(lua: &'static Lua) -> mlua::Result<()> {
		let ps = lua.create_table()?;

		ps.raw_set(
			"pub",
			lua.create_function(|_, (kind, value): (mlua::String, Value)| {
				yazi_dds::Pubsub::pub_(Body::from_lua(kind.to_str()?, value)?);
				Ok(())
			})?,
		)?;

		ps.raw_set(
			"pub_to",
			lua.create_function(|_, (receiver, kind, value): (u64, mlua::String, Value)| {
				yazi_dds::Pubsub::pub_to(receiver, Body::from_lua(kind.to_str()?, value)?);
				Ok(())
			})?,
		)?;

		ps.raw_set(
			"sub",
			lua.create_function(|lua, (kind, f): (mlua::String, Function)| {
				let rt = lua.named_registry_value::<RtRef>("rt")?;
				let Some(cur) = rt.current() else {
					return Err("`sub()` must be called in a sync plugin").into_lua_err();
				};
				if !yazi_dds::Pubsub::sub(cur, kind.to_str()?, f) {
					return Err("`sub()` called twice").into_lua_err();
				}
				Ok(())
			})?,
		)?;

		ps.raw_set(
			"sub_remote",
			lua.create_function(|_, (kind, f): (mlua::String, Function)| {
				let rt = lua.named_registry_value::<RtRef>("rt")?;
				let Some(cur) = rt.current() else {
					return Err("`sub_remote()` must be called in a sync plugin").into_lua_err();
				};
				if !yazi_dds::Pubsub::sub_remote(cur, kind.to_str()?, f) {
					return Err("`sub_remote()` called twice").into_lua_err();
				}
				Ok(())
			})?,
		)?;

		ps.raw_set(
			"unsub",
			lua.create_function(|_, kind: mlua::String| {
				if let Some(cur) = lua.named_registry_value::<RtRef>("rt")?.current() {
					Ok(yazi_dds::Pubsub::unsub(cur, kind.to_str()?))
				} else {
					Err("`unsub()` must be called in a sync plugin").into_lua_err()
				}
			})?,
		)?;

		ps.raw_set(
			"unsub_remote",
			lua.create_function(|_, kind: mlua::String| {
				if let Some(cur) = lua.named_registry_value::<RtRef>("rt")?.current() {
					Ok(yazi_dds::Pubsub::unsub_remote(cur, kind.to_str()?))
				} else {
					Err("`unsub_remote()` must be called in a sync plugin").into_lua_err()
				}
			})?,
		)?;

		lua.globals().raw_set("ps", ps)
	}
}
