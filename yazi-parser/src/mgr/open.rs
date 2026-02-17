use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlCow};

#[derive(Clone, Debug)]
pub struct OpenOpt {
	pub cwd:         Option<UrlCow<'static>>,
	pub targets:     Vec<UrlCow<'static>>,
	pub interactive: bool,
	pub hovered:     bool,
}

impl TryFrom<ActionCow> for OpenOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		if let Some(opt) = a.take_any2("opt") {
			return opt;
		}

		Ok(Self {
			cwd:         a.take("cwd").ok(),
			targets:     a.take_seq(),
			interactive: a.bool("interactive"),
			hovered:     a.bool("hovered"),
		})
	}
}

impl FromLua for OpenOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for OpenOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
