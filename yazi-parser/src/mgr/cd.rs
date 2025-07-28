use std::borrow::Cow;

use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_fs::expand_url;
use yazi_shared::{event::CmdCow, url::Url};

#[derive(Debug)]
pub struct CdOpt {
	pub target:      Url,
	pub interactive: bool,
	pub source:      CdSource,
}

impl From<CmdCow> for CdOpt {
	fn from(mut c: CmdCow) -> Self {
		let mut target = c.take_first_url().unwrap_or_default();

		if !c.bool("raw")
			&& let Cow::Owned(u) = expand_url(&target)
		{
			target = u;
		}

		Self { target, interactive: c.bool("interactive"), source: CdSource::Cd }
	}
}

impl From<(Url, CdSource)> for CdOpt {
	fn from((target, source): (Url, CdSource)) -> Self { Self { target, interactive: false, source } }
}

impl IntoLua for &CdOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// --- Source
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CdSource {
	Tab,
	Cd,
	Reveal,
	Enter,
	Leave,
	Forward,
	Back,
}

impl CdSource {
	#[inline]
	pub fn big_jump(self) -> bool { self == Self::Cd || self == Self::Reveal }
}
