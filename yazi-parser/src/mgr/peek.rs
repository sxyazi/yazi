use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::{CmdCow, Data}, url::UrlCow};

#[derive(Debug, Default)]
pub struct PeekOpt {
	pub skip:        Option<usize>,
	pub force:       bool,
	pub only_if:     Option<UrlCow<'static>>,
	pub upper_bound: bool,
	pub cycle:       Option<i32>,
}

impl From<CmdCow> for PeekOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			skip:        c.first().and_then(Data::as_usize),
			force:       c.bool("force"),
			only_if:     c.take_url("only-if"),
			upper_bound: c.bool("upper-bound"),
			cycle:       c.take_str("cycle").and_then(|s| parse_cycle(s.as_ref())),
		}
	}
}

impl From<bool> for PeekOpt {
	fn from(force: bool) -> Self { Self { force, ..Default::default() } }
}

impl FromLua for PeekOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for PeekOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

fn parse_cycle(value: &str) -> Option<i32> {
	match value.to_ascii_lowercase().as_str() {
		"next" | "forward" | "+" => Some(1),
		"prev" | "previous" | "back" | "-" => Some(-1),
		"reset" | "first" | "start" | "0" => Some(0),
		_ => value.parse().ok(),
	}
}
