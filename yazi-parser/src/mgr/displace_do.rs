use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlBuf};

#[derive(Clone, Debug)]
pub struct DisplaceDoOpt {
	pub to:   Result<UrlBuf, yazi_fs::error::Error>,
	pub from: UrlBuf,
}

impl TryFrom<ActionCow> for DisplaceDoOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		if let Some(opt) = a.take_any2("opt") {
			opt
		} else {
			bail!("Invalid 'opt' in DisplaceDoOpt");
		}
	}
}

impl FromLua for DisplaceDoOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for DisplaceDoOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
