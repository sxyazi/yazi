use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Table, Value};
use serde::{Deserialize, Serialize};
use yazi_core::mgr::CdSource;
use yazi_shared::{event::{ActionCow, FromAction}, url::UrlBuf};
use yazi_shim::mlua::SER_OPT;

use crate::mgr::CdForm;

#[derive(Debug, Deserialize, Serialize)]
pub struct StashForm {
	#[serde(alias = "0")]
	pub target: UrlBuf,
	source:     CdSource,
}

impl<C> FromAction<C> for StashForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl From<&CdForm> for StashForm {
	fn from(form: &CdForm) -> Self { Self { target: form.target.clone(), source: form.source } }
}

impl FromLua for StashForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self { target: t.raw_get("target")?, source: lua.from_value(t.raw_get("source")?)? })
	}
}

impl IntoLua for StashForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("target", self.target.into_lua(lua)?),
				("source", lua.to_value_with(&self.source, SER_OPT)?),
			])?
			.into_lua(lua)
	}
}
