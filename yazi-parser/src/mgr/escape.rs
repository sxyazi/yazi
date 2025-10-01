use bitflags::bitflags;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

bitflags! {
	#[derive(Debug)]
	pub struct EscapeOpt: u8 {
		const FIND   = 0b00001;
		const VISUAL = 0b00010;
		const FILTER = 0b00100;
		const SELECT = 0b01000;
		const SEARCH = 0b10000;
	}
}

impl From<CmdCow> for EscapeOpt {
	fn from(c: CmdCow) -> Self {
		c.args.iter().fold(Self::empty(), |acc, (k, v)| {
			match (k.as_str().unwrap_or(""), v.try_into().unwrap_or(false)) {
				("all", true) => Self::all(),
				("find", true) => acc | Self::FIND,
				("visual", true) => acc | Self::VISUAL,
				("filter", true) => acc | Self::FILTER,
				("select", true) => acc | Self::SELECT,
				("search", true) => acc | Self::SEARCH,
				_ => acc,
			}
		})
	}
}

impl FromLua for EscapeOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for EscapeOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
