use bitflags::bitflags;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

bitflags! {
	#[derive(Debug)]
	pub struct EscapeForm: u8 {
		const FIND   = 1 << 0;
		const VISUAL = 1 << 1;
		const FILTER = 1 << 2;
		const SELECT = 1 << 3;
		const VIEW   = 1 << 4;
	}
}

impl From<ActionCow> for EscapeForm {
	fn from(a: ActionCow) -> Self {
		a.args.iter().fold(Self::empty(), |acc, (k, v)| {
			match (k.as_str().unwrap_or(""), v.try_into().unwrap_or(false)) {
				("all", true) => Self::all(),
				("find", true) => acc | Self::FIND,
				("visual", true) => acc | Self::VISUAL,
				("filter", true) => acc | Self::FILTER,
				("select", true) => acc | Self::SELECT,
				("view", true) => acc | Self::VIEW,
				_ => acc,
			}
		})
	}
}

impl FromLua for EscapeForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for EscapeForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
