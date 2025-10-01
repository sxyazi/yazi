use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::FilterCase;
use yazi_shared::{SStr, event::CmdCow};

#[derive(Debug, Default)]
pub struct FilterOpt {
	pub query: SStr,
	pub case:  FilterCase,
	pub done:  bool,
}

impl TryFrom<CmdCow> for FilterOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			return opt;
		}

		Ok(Self {
			query: c.take_first().unwrap_or_default(),
			case:  FilterCase::from(&*c),
			done:  c.bool("done"),
		})
	}
}

impl FromLua for FilterOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for FilterOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
