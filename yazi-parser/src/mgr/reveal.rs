use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::path::expand_url;
use yazi_shared::{event::CmdCow, url::{UrlBuf, UrlCow}};

use crate::mgr::CdSource;

#[derive(Debug)]
pub struct RevealOpt {
	pub target:   UrlCow<'static>,
	pub source:   CdSource,
	pub no_dummy: bool,
}

impl From<CmdCow> for RevealOpt {
	fn from(mut c: CmdCow) -> Self {
		let mut target = c.take_first_url().unwrap_or_default();

		if !c.bool("raw") {
			target = expand_url(target).into();
		}

		Self { target, source: CdSource::Reveal, no_dummy: c.bool("no-dummy") }
	}
}

impl From<UrlCow<'static>> for RevealOpt {
	fn from(target: UrlCow<'static>) -> Self {
		Self { target, source: CdSource::Reveal, no_dummy: false }
	}
}

impl From<(UrlCow<'static>, CdSource)> for RevealOpt {
	fn from((target, source): (UrlCow<'static>, CdSource)) -> Self {
		Self { target, source, no_dummy: false }
	}
}

impl From<UrlBuf> for RevealOpt {
	fn from(target: UrlBuf) -> Self { Self::from(UrlCow::from(target)) }
}

impl From<(UrlBuf, CdSource)> for RevealOpt {
	fn from((target, source): (UrlBuf, CdSource)) -> Self {
		Self::from((UrlCow::from(target), source))
	}
}

impl FromLua for RevealOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RevealOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
