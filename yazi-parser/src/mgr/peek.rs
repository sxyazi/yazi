use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_shared::{event::{CmdCow, Data}, url::Url};

#[derive(Debug, Default)]
pub struct PeekOpt {
	pub skip:        Option<usize>,
	pub force:       bool,
	pub only_if:     Option<Url>,
	pub upper_bound: bool,
}

impl From<CmdCow> for PeekOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			skip:        c.first().and_then(Data::as_usize),
			force:       c.bool("force"),
			only_if:     c.take_url("only-if"),
			upper_bound: c.bool("upper-bound"),
		}
	}
}

impl From<bool> for PeekOpt {
	fn from(force: bool) -> Self { Self { force, ..Default::default() } }
}

impl IntoLua for &PeekOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
