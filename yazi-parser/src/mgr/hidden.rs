use std::str::FromStr;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::event::CmdCow;

#[derive(Debug, Default)]
pub struct HiddenOpt {
	pub state: HiddenOptState,
}

impl TryFrom<CmdCow> for HiddenOpt {
	type Error = anyhow::Error;

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { state: c.str(0).parse().unwrap_or_default() })
	}
}

impl FromLua for HiddenOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for HiddenOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// --- State
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HiddenOptState {
	#[default]
	None,
	Show,
	Hide,
	Toggle,
}

impl FromStr for HiddenOptState {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}

impl HiddenOptState {
	pub fn bool(self, old: bool) -> bool {
		match self {
			Self::None => old,
			Self::Show => true,
			Self::Hide => false,
			Self::Toggle => !old,
		}
	}
}
