use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlBuf};

#[derive(Debug)]
pub struct UpdateSucceedOpt {
	pub urls: Vec<UrlBuf>,
}

impl TryFrom<ActionCow> for UpdateSucceedOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Some(urls) = a.take_any("urls") else {
			bail!("Invalid 'urls' in UpdateSucceedOpt");
		};

		Ok(Self { urls })
	}
}

impl FromLua for UpdateSucceedOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateSucceedOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
