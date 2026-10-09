use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_fs::{SortBy, SortFallback};
use yazi_shared::event::{ActionCow, FromAction};
use yazi_shim::mlua::SER_OPT;

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

impl<C> FromAction<C> for SortForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for SortForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for SortForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
