use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_macro::impl_data_any;
use yazi_shared::{event::{ActionCow, FromAction}, strand::StrandBuf};
use yazi_shim::mlua::SER_OPT;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct QuitOpt {
	#[serde(default)]
	pub code:        i32,
	#[serde(skip)]
	pub selected:    Option<StrandBuf>,
	#[serde(default, alias = "no-cwd-file")]
	pub no_cwd_file: bool,
}

impl_data_any!(QuitOpt);

impl<C> FromAction<C> for QuitOpt {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for QuitOpt {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for QuitOpt {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
