use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_config::keymap::Chord;
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct ShowOpt {
	pub cands:  Vec<Chord>,
	pub silent: bool,
}

impl TryFrom<CmdCow> for ShowOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			return opt;
		}

		Ok(Self { cands: c.take_any("candidates").unwrap_or_default(), silent: c.bool("silent") })
	}
}

impl FromLua for ShowOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ShowOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
