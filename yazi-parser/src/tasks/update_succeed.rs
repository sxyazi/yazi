use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{Id, event::ActionCow, url::UrlBuf};

#[derive(Debug)]
pub struct UpdateSucceedForm {
	pub id:    Id,
	pub urls:  Vec<UrlBuf>,
	pub track: bool,
}

impl TryFrom<ActionCow> for UpdateSucceedForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Some(urls) = a.take_any("urls") else {
			bail!("Invalid 'urls' in UpdateSucceedForm");
		};

		Ok(Self { id: a.first()?, urls, track: a.get("track").unwrap_or_default() })
	}
}

impl FromLua for UpdateSucceedForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateSucceedForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
