use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlCow};

#[derive(Debug)]
pub struct BulkExitOpt {
	pub target: UrlCow<'static>,
	pub accept: bool,
}

impl TryFrom<ActionCow> for BulkExitOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Ok(target) = a.take_first::<UrlCow>() else {
			bail!("invalid target in BulkExitOpt");
		};

		Ok(Self { target, accept: a.bool("accept") })
	}
}

impl FromLua for BulkExitOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for BulkExitOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
