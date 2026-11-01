use std::{ops::{Deref, DerefMut}, path::MAIN_SEPARATOR_STR};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::{ActionCow, FromAction}, id::Id, strand::StrandLike};

use crate::cmp::CmpItem;

#[derive(Debug)]
pub struct CompleteOpt {
	pub item:   CmpItem,
	pub ticket: Id,
}

impl Deref for CompleteOpt {
	type Target = CmpItem;

	fn deref(&self) -> &Self::Target { &self.item }
}

impl DerefMut for CompleteOpt {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.item }
}

impl<C> FromAction<C> for CompleteOpt {
	fn from_action(a: ActionCow, cx: &C) -> anyhow::Result<Self> {
		let ticket = a.get("ticket").unwrap_or_default();

		Ok(Self { item: CmpItem::from_action(a, cx)?, ticket })
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
		format!("{}{}", self.name.display(), if self.is_dir { MAIN_SEPARATOR_STR } else { "" })
	}
}
