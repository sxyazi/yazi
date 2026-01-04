use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_binding::Url;
use yazi_shared::{event::CmdCow, url::UrlBuf};

use crate::mgr::{CdOpt, CdSource};

#[derive(Debug, Deserialize, Serialize)]
pub struct StashOpt {
	pub target: UrlBuf,
	pub source: CdSource,
}

impl TryFrom<CmdCow> for StashOpt {
	type Error = anyhow::Error;

	fn try_from(_: CmdCow) -> Result<Self, Self::Error> { bail!("unsupported") }
}

impl From<CdOpt> for StashOpt {
	fn from(opt: CdOpt) -> Self { Self { target: opt.target, source: opt.source } }
}

impl FromLua for StashOpt {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let tbl = value.as_table().ok_or_else(|| "expected table".into_lua_err())?;
		Ok(Self {
			target: tbl.get::<Url>("target")?.into(),
			source: lua.from_value(tbl.get("source")?)?,
		})
	}
}

impl IntoLua for StashOpt {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("target", Url::new(self.target).into_lua(lua)?),
				("source", lua.to_value(&self.source)?),
			])?
			.into_lua(lua)
	}
}
