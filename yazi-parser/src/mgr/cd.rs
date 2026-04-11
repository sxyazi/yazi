use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_core::mgr::CdSource;
use yazi_fs::path::{clean_url, expand_url};
use yazi_shared::{event::ActionCow, url::{Url, UrlBuf}};
use yazi_vfs::provider;

#[derive(Debug, Deserialize)]
pub struct CdForm {
	#[serde(alias = "0", default)]
	pub target:      UrlBuf,
	#[serde(default)]
	pub interactive: bool,
	#[serde(default)]
	pub raw:         bool,
	#[serde(default)]
	pub source:      CdSource,
}

impl TryFrom<ActionCow> for CdForm {
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

impl From<(UrlBuf, CdSource)> for CdForm {
	fn from((target, source): (UrlBuf, CdSource)) -> Self {
		Self { target, interactive: false, raw: false, source }
	}
}

impl From<(Url<'_>, CdSource)> for CdForm {
	fn from((target, source): (Url, CdSource)) -> Self { Self::from((target.to_owned(), source)) }
}

impl FromLua for CdForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CdForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
