use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_boot::BOOT;
use yazi_fs::path::{clean_url, expand_url};
use yazi_shared::{event::ActionCow, url::UrlBuf};
use yazi_vfs::provider;

#[derive(Debug, Deserialize)]
pub struct TabCreateForm {
	#[serde(alias = "0")]
	pub target:  Option<UrlBuf>,
	#[serde(default)]
	pub current: bool,
	#[serde(default)]
	pub raw:     bool,
}

impl TryFrom<ActionCow> for TabCreateForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		let mut me: Self = a.deserialize()?;

		if me.current {
			me.target = None;
		} else if me.target.is_none() {
			me.target = Some(BOOT.cwds[0].clone());
		} else if let Some(mut target) = me.target {
			if !me.raw {
				target = expand_url(target).into_owned();
			}

			if let Some(u) = provider::try_absolute(&target)
				&& u.is_owned()
			{
				target = u.into_owned();
			}

			me.target = Some(clean_url(target));
		}

		Ok(me)
	}
}

impl FromLua for TabCreateForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TabCreateForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
