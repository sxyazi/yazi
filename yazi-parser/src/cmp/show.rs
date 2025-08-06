use std::{ffi::OsString, path::MAIN_SEPARATOR_STR};

use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{Id, event::CmdCow, url::{Url, UrnBuf}};

#[derive(Debug, Default)]
pub struct ShowOpt {
	pub cache:      Vec<CmpItem>,
	pub cache_name: Url,
	pub word:       UrnBuf,
	pub ticket:     Id,
}

impl TryFrom<CmdCow> for ShowOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
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
	pub name:   OsString,
	pub is_dir: bool,
}

impl CmpItem {
	pub fn completable(&self) -> String {
		format!("{}{}", self.name.to_string_lossy(), if self.is_dir { MAIN_SEPARATOR_STR } else { "" })
	}
}
