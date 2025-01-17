use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyMount;

impl BodyMount {
	#[inline]
	pub fn owned() -> Body<'static> { Self.into() }
}

impl From<BodyMount> for Body<'_> {
	fn from(value: BodyMount) -> Self { Self::Mount(value) }
}

impl IntoLua for BodyMount {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Ok(Value::Nil) }
}
