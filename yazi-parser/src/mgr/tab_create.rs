use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_boot::BOOT;
use yazi_fs::path::expand_url;
use yazi_shared::{event::CmdCow, url::UrlCow};

#[derive(Debug)]
pub struct TabCreateOpt {
	pub wd: Option<UrlCow<'static>>,
}

impl From<CmdCow> for TabCreateOpt {
	fn from(mut c: CmdCow) -> Self {
		if c.bool("current") {
			return Self { wd: None };
		}
		let Some(mut wd) = c.take_first_url() else {
			return Self { wd: Some(UrlCow::from(&BOOT.cwds[0])) };
		};
		if !c.bool("raw") {
			wd = expand_url(wd).into();
		}
		Self { wd: Some(wd) }
	}
}

impl FromLua for TabCreateOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TabCreateOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
