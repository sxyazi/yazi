use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Body;
use crate::ValueSendable;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyCustom {
	#[serde(skip)]
	pub kind:  String,
	#[serde(flatten)]
	pub value: ValueSendable,
}

impl BodyCustom {
	#[inline]
	pub fn from_str(kind: &str, value: &str) -> anyhow::Result<Body<'static>> {
		let mut me = serde_json::from_str::<Self>(value)?;
		kind.clone_into(&mut me.kind);
		Ok(me.into())
	}

	#[inline]
	pub fn from_lua(kind: &str, value: Value) -> mlua::Result<Body<'static>> {
		Ok(Self { kind: kind.to_owned(), value: value.try_into()? }.into())
	}
}

impl From<BodyCustom> for Body<'_> {
	fn from(value: BodyCustom) -> Self { Self::Custom(value) }
}

impl IntoLua<'_> for BodyCustom {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.value.into_lua(lua) }
}
