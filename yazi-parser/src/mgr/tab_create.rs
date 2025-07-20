use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_boot::BOOT;
use yazi_fs::expand_path;
use yazi_shared::{event::CmdCow, url::Url};

#[derive(Debug)]
pub struct TabCreateOpt {
	pub wd: Option<Url>,
}

impl From<CmdCow> for TabCreateOpt {
	fn from(mut c: CmdCow) -> Self {
		if c.bool("current") {
			return Self { wd: None };
		}
		let Some(mut wd) = c.take_first_url() else {
			return Self { wd: Some(Url::from(&BOOT.cwds[0])) };
		};
		if wd.is_regular() && !c.bool("raw") {
			wd = Url::from(expand_path(wd));
		}
		Self { wd: Some(wd) }
	}
}

impl IntoLua for &TabCreateOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
