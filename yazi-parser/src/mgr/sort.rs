use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_fs::SortBy;
use yazi_shared::event::CmdCow;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SortOpt {
	pub by:        Option<SortBy>,
	pub reverse:   Option<bool>,
	pub dir_first: Option<bool>,
	pub sensitive: Option<bool>,
	pub translit:  Option<bool>,
}

impl TryFrom<CmdCow> for SortOpt {
	type Error = anyhow::Error;

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self {
			by:        c.first().ok().map(str::parse).transpose()?,
			reverse:   c.get("reverse").ok(),
			dir_first: c.get("dir-first").ok(),
			sensitive: c.get("sensitive").ok(),
			translit:  c.get("translit").ok(),
		})
	}
}

impl FromLua for SortOpt {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for SortOpt {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value(&self) }
}
