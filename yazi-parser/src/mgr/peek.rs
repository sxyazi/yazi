use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlBuf};

#[derive(Debug, Default)]
pub struct PeekForm {
	pub skip:        Option<usize>,
	pub force:       bool,
	pub only_if:     Option<UrlBuf>,
	pub upper_bound: bool,
}

impl From<ActionCow> for PeekForm {
	fn from(mut a: ActionCow) -> Self {
		Self {
			skip:        a.first().ok(),
			force:       a.bool("force"),
			only_if:     a.take("only-if").ok(),
			upper_bound: a.bool("upper-bound"),
		}
	}
}

impl From<bool> for PeekForm {
	fn from(force: bool) -> Self { Self { force, ..Default::default() } }
}

impl FromLua for PeekForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for PeekForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
