use hashbrown::HashMap;
use mlua::{ExternalError, FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::event::ActionCow;
use yazi_shim::mlua::SER_OPT;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoadHistoryForm {
	#[serde(alias = "0")]
	pub entries: HashMap<String, Vec<String>>,
}

impl TryFrom<ActionCow> for LoadHistoryForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for LoadHistoryForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> {
		Err("unexpected LoadHistoryForm from Lua".into_lua_err())
	}
}

impl IntoLua for LoadHistoryForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self.entries, SER_OPT) }
}
