use std::path::MAIN_SEPARATOR_STR;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::{ActionCow, FromAction}, id::Id, strand::{StrandBuf, StrandLike}};

#[derive(Debug)]
pub struct CompleteOpt {
	pub name:   StrandBuf,
	pub is_dir: bool,
	pub ticket: Id,
}

impl<C> FromAction<C> for CompleteOpt {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self {
			name:   a.take("name")?,
			is_dir: a.bool("is_dir"),
			ticket: a.get("ticket").unwrap_or_default(),
		})
	}
}

impl FromLua for CompleteOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CompleteOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

impl CompleteOpt {
	pub(crate) fn completable(&self) -> String {
		format!("{}{}", self.name.to_string_lossy(), if self.is_dir { MAIN_SEPARATOR_STR } else { "" })
	}
}
