use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::SortBy;
use yazi_shared::event::CmdCow;

#[derive(Debug, Default)]
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
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for SortOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
