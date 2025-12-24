use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_fs::path::{clean_url, expand_url};
use yazi_shared::{event::CmdCow, url::{Url, UrlBuf}};
use yazi_vfs::provider;

#[derive(Debug)]
pub struct CdOpt {
	pub target:      UrlBuf,
	pub interactive: bool,
	pub source:      CdSource,
}

impl From<CmdCow> for CdOpt {
	fn from(mut c: CmdCow) -> Self {
		let mut target = c.take_first().unwrap_or_default();

		if !c.bool("raw") {
			target = expand_url(target);
		}

		if let Some(u) = provider::try_absolute(&target)
			&& u.is_owned()
		{
			target = u.into_static();
		}

		Self {
			target:      clean_url(target),
			interactive: c.bool("interactive"),
			source:      CdSource::Cd,
		}
	}
}

impl From<(UrlBuf, CdSource)> for CdOpt {
	fn from((target, source): (UrlBuf, CdSource)) -> Self {
		Self { target, interactive: false, source }
	}
}

impl From<(Url<'_>, CdSource)> for CdOpt {
	fn from((target, source): (Url, CdSource)) -> Self { Self::from((target.to_owned(), source)) }
}

impl FromLua for CdOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CdOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// --- Source
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CdSource {
	Tab,
	Cd,
	Reveal,
	Enter,
	Leave,
	Follow,
	Forward,
	Back,
	Escape,
	Displace,
}
