use std::borrow::Cow;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::expand_url;
use yazi_shared::{event::CmdCow, url::Url};

use crate::mgr::CdSource;

#[derive(Debug)]
pub struct RevealOpt {
	pub target:   Url,
	pub source:   CdSource,
	pub no_dummy: bool,
}

impl From<CmdCow> for RevealOpt {
	fn from(mut c: CmdCow) -> Self {
		let mut target = c.take_first_url().unwrap_or_default();

		if !c.bool("raw")
			&& let Cow::Owned(u) = expand_url(&target)
		{
			target = u;
		}

		Self { target, source: CdSource::Reveal, no_dummy: c.bool("no-dummy") }
	}
}

impl From<Url> for RevealOpt {
	fn from(target: Url) -> Self { Self { target, source: CdSource::Reveal, no_dummy: false } }
}

impl From<(Url, CdSource)> for RevealOpt {
	fn from((target, source): (Url, CdSource)) -> Self { Self { target, source, no_dummy: false } }
}

impl FromLua for RevealOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RevealOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
