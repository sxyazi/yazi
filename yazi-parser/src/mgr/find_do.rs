use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::FilterCase;
use yazi_shared::{SStr, event::CmdCow};

#[derive(Debug)]
pub struct FindDoOpt {
	pub query: SStr,
	pub prev:  bool,
	pub case:  FilterCase,
}

impl TryFrom<CmdCow> for FindDoOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			return opt;
		}

		let Ok(query) = c.take_first() else {
			bail!("'query' is required for FindDoOpt");
		};

		Ok(Self { query, prev: c.bool("previous"), case: FilterCase::from(&*c) })
	}
}

impl FromLua for FindDoOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for FindDoOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
