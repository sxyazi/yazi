use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::path::expand_url;
use yazi_shared::{event::CmdCow, url::{Url, UrlBuf, UrlCow}};

#[derive(Debug)]
pub struct CdOpt {
	pub target:      UrlCow<'static>,
	pub interactive: bool,
	pub source:      CdSource,
}

impl From<CmdCow> for CdOpt {
	fn from(mut c: CmdCow) -> Self {
		let mut target = c.take_first_url().unwrap_or_default();

		if !c.bool("raw") {
			target = expand_url(target).into();
		}

		Self { target, interactive: c.bool("interactive"), source: CdSource::Cd }
	}
}

impl From<(UrlCow<'static>, CdSource)> for CdOpt {
	fn from((target, source): (UrlCow<'static>, CdSource)) -> Self {
		Self { target, interactive: false, source }
	}
}

impl From<(UrlBuf, CdSource)> for CdOpt {
	fn from((target, source): (UrlBuf, CdSource)) -> Self {
		Self::from((UrlCow::from(target), source))
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
