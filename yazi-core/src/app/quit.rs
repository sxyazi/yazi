use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_binding::SER_OPT;
use yazi_shared::{event::ActionCow, strand::StrandBuf};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct QuitOpt {
	pub code:        i32,
	#[serde(skip)]
	pub selected:    Option<StrandBuf>,
	pub no_cwd_file: bool,
}

impl TryFrom<ActionCow> for QuitOpt {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self {
			code:        a.get("code").unwrap_or_default(),
			selected:    None,
			no_cwd_file: a.bool("no-cwd-file"),
		})
	}
}

impl FromLua for QuitOpt {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for QuitOpt {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
