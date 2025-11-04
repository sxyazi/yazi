use mlua::{IntoLua, Lua, Value};
use yazi_binding::{Composer, ComposerGet, ComposerSet};

yazi_macro::mod_flat!(pubsub);

pub(super) fn compose() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
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
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}
