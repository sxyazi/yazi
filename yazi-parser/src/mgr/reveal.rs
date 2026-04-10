use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_core::mgr::CdSource;
use yazi_fs::path::{clean_url, expand_url};
use yazi_shared::{event::ActionCow, url::UrlBuf};
use yazi_vfs::provider;

#[derive(Debug, Deserialize)]
pub struct RevealForm {
	#[serde(alias = "0")]
	pub target:   UrlBuf,
	#[serde(default)]
	pub raw:      bool,
	#[serde(default = "default_source")]
	pub source:   CdSource,
	#[serde(alias = "no-dummy", default)]
	pub no_dummy: bool,
}

impl TryFrom<ActionCow> for RevealForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		let mut me: Self = a.deserialize()?;

		if !me.raw {
			me.target = expand_url(me.target).into_owned();
		}

		if let Some(u) = provider::try_absolute(&me.target)
			&& u.is_owned()
		{
			me.target = u.into_owned();
		}

		me.target = clean_url(me.target);
		Ok(me)
	}
}

impl From<UrlBuf> for RevealForm {
	fn from(target: UrlBuf) -> Self {
		Self { target, raw: false, source: CdSource::Reveal, no_dummy: false }
	}
}

impl From<(UrlBuf, CdSource)> for RevealForm {
	fn from((target, source): (UrlBuf, CdSource)) -> Self {
		Self { target, raw: false, source, no_dummy: false }
	}
}

impl FromLua for RevealForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RevealForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

fn default_source() -> CdSource { CdSource::Reveal }
