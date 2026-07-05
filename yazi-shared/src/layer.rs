use mlua::{ExternalError, ExternalResult, FromLua, MetaMethod, UserData, UserDataMethods, Value};
use serde::Deserialize;
use strum::{Display, EnumString, FromRepr, IntoStaticStr};
use yazi_shim::strum::IntoStr;

#[derive(
	Clone,
	Copy,
	Debug,
	Default,
	Deserialize,
	Display,
	EnumString,
	Eq,
	FromRepr,
	Hash,
	IntoStaticStr,
	PartialEq,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
pub enum Layer {
	#[default]
	Null,
	App,
	Mgr,
	Tasks,
	Spot,
	Pick,
	Input,
	Confirm,
	Help,
	Cmp,
	Which,
	Notify,
}

impl Layer {
	pub fn or(self, other: Self) -> Self { if self == Self::Null { other } else { self } }
}

impl FromLua for Layer {
	fn from_lua(value: Value, _: &mlua::Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::String(s) => s.to_str()?.parse().into_lua_err()?,
			Value::UserData(ud) => *ud.borrow::<Self>()?,
			_ => Err("expected a string or a Layer".into_lua_err())?,
		})
	}
}

impl UserData for Layer {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.into_str()));
	}
}
