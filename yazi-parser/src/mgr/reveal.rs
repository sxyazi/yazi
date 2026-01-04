use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::path::{clean_url, expand_url};
use yazi_shared::{event::CmdCow, url::UrlBuf};
use yazi_vfs::provider;

use crate::mgr::CdSource;

#[derive(Debug)]
pub struct RevealOpt {
	pub target:   UrlBuf,
	pub source:   CdSource,
	pub no_dummy: bool,
}

impl From<CmdCow> for RevealOpt {
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

		Self { target: clean_url(target), source: CdSource::Reveal, no_dummy: c.bool("no-dummy") }
	}
}

impl From<UrlBuf> for RevealOpt {
	fn from(target: UrlBuf) -> Self { Self { target, source: CdSource::Reveal, no_dummy: false } }
}

impl From<(UrlBuf, CdSource)> for RevealOpt {
	fn from((target, source): (UrlBuf, CdSource)) -> Self { Self { target, source, no_dummy: false } }
}

impl FromLua for RevealOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RevealOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
