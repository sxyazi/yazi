use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_boot::BOOT;
use yazi_fs::path::{clean_url, expand_url};
use yazi_shared::{event::ActionCow, url::UrlCow};
use yazi_vfs::provider;

#[derive(Debug)]
pub struct TabCreateOpt {
	pub url: Option<UrlCow<'static>>,
}

impl From<ActionCow> for TabCreateOpt {
	fn from(mut a: ActionCow) -> Self {
		if a.bool("current") {
			return Self { url: None };
		}

		let Ok(mut url) = a.take_first() else {
			return Self { url: Some(UrlCow::from(&BOOT.cwds[0])) };
		};

		if !a.bool("raw") {
			url = expand_url(url);
		}

		if let Some(u) = provider::try_absolute(&url)
			&& u.is_owned()
		{
			url = u.into_static();
		}

		Self { url: Some(clean_url(url).into()) }
	}
}

impl FromLua for TabCreateOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TabCreateOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
