use mlua::{ExternalError, FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_binding::{SER_OPT, Url};
use yazi_core::mgr::CdSource;
use yazi_shared::{event::ActionCow, url::UrlBuf};

use crate::mgr::CdForm;

#[derive(Debug, Deserialize, Serialize)]
pub struct StashForm {
	#[serde(alias = "0")]
	pub target: UrlBuf,
	pub source: CdSource,
}

impl TryFrom<ActionCow> for StashForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl From<&CdForm> for StashForm {
	fn from(form: &CdForm) -> Self { Self { target: form.target.clone(), source: form.source } }
}

impl FromLua for StashForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let tbl = value.as_table().ok_or_else(|| "expected table".into_lua_err())?;
		Ok(Self {
			target: tbl.get::<Url>("target")?.into(),
			source: lua.from_value(tbl.get("source")?)?,
		})
	}
}

impl IntoLua for StashForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("target", Url::new(self.target).into_lua(lua)?),
				("source", lua.to_value_with(&self.source, SER_OPT)?),
			])?
			.into_lua(lua)
	}
}
