use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_binding::SER_OPT;
use yazi_fs::{SortBy, SortFallback};
use yazi_shared::event::ActionCow;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SortOpt {
	pub by:        Option<SortBy>,
	pub reverse:   Option<bool>,
	pub dir_first: Option<bool>,
	pub sensitive: Option<bool>,
	pub translit:  Option<bool>,
	pub fallback:  Option<SortFallback>,
}

impl TryFrom<ActionCow> for SortOpt {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self {
			by:        a.first().ok().map(str::parse).transpose()?,
			reverse:   a.get("reverse").ok(),
			dir_first: a.get("dir-first").ok(),
			sensitive: a.get("sensitive").ok(),
			translit:  a.get("translit").ok(),
			fallback:  a.get("fallback").ok().map(str::parse).transpose()?,
		})
	}
}

impl FromLua for SortOpt {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for SortOpt {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
