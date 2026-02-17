use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{Id, event::ActionCow, path::PathBufDyn, strand::StrandBuf, url::UrlBuf};

#[derive(Clone, Debug)]
pub struct ShowOpt {
	pub cache:      Vec<CmpItem>,
	pub cache_name: UrlBuf,
	pub word:       PathBufDyn,
	pub ticket:     Id,
}

impl TryFrom<ActionCow> for ShowOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		if let Some(opt) = a.take_any2("opt") {
			opt
		} else {
			bail!("missing 'opt' argument");
		}
	}
}

impl FromLua for ShowOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ShowOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// --- Item
#[derive(Debug, Clone)]
pub struct CmpItem {
	pub name:   StrandBuf,
	pub is_dir: bool,
}
