use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlCow};

#[derive(Debug, Default)]
pub struct PeekOpt {
	pub skip:        Option<usize>,
	pub force:       bool,
	pub only_if:     Option<UrlCow<'static>>,
	pub upper_bound: bool,
}

impl From<ActionCow> for PeekOpt {
	fn from(mut a: ActionCow) -> Self {
		Self {
			skip:        a.first().ok(),
			force:       a.bool("force"),
			only_if:     a.take("only-if").ok(),
			upper_bound: a.bool("upper-bound"),
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
