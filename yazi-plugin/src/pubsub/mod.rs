#![allow(clippy::module_inception)]

use mlua::{IntoLua, Lua, Value};

use crate::Composer;

yazi_macro::mod_flat!(pubsub);

pub(super) fn compose(lua: &Lua) -> mlua::Result<Value> {
	Composer::make(lua, |lua, key| {
		match key {
			b"pub" => Pubsub::r#pub(lua)?,
			b"pub_to" => Pubsub::pub_to(lua)?,
			b"sub" => Pubsub::sub(lua)?,
			b"sub_remote" => Pubsub::sub_remote(lua)?,
			b"unsub" => Pubsub::unsub(lua)?,
			b"unsub_remote" => Pubsub::unsub_remote(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	})
}
