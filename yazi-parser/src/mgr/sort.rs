use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_binding::SER_OPT;
use yazi_fs::{SortBy, SortFallback};
use yazi_shared::event::ActionCow;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SortForm {
	#[serde(alias = "0")]
	pub by:        Option<SortBy>,
	pub reverse:   Option<bool>,
	#[serde(alias = "dir-first")]
	pub dir_first: Option<bool>,
	pub sensitive: Option<bool>,
	pub translit:  Option<bool>,
	pub fallback:  Option<SortFallback>,
}

impl TryFrom<ActionCow> for SortForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for SortForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for SortForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
