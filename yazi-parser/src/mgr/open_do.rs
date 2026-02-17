use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlCow};

#[derive(Clone, Debug, Default)]
pub struct OpenDoOpt {
	pub cwd:         UrlCow<'static>,
	pub targets:     Vec<UrlCow<'static>>,
	pub interactive: bool,
}

impl TryFrom<ActionCow> for OpenDoOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		if let Some(opt) = a.take_any2("opt") {
			opt
		} else {
			bail!("Invalid 'opt' in OpenDoOpt");
		}
	}
}

impl FromLua for OpenDoOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for OpenDoOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
