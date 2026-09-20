use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_fs::op::FilesOp;
use yazi_shared::id::Id;

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberPatch<'a> {
	tab: Id,
	#[serde(flatten)]
	op:  Cow<'a, FilesOp>,
}

impl<'a> EmberPatch<'a> {
	pub(crate) fn borrowed(tab: Id, op: &'a FilesOp) -> Ember<'a> {
		Self { tab, op: Cow::Borrowed(op) }.into()
	}
}

impl EmberPatch<'static> {
	pub(crate) fn owned(tab: Id, op: &FilesOp) -> Ember<'static> {
		Self { tab, op: Cow::Owned(op.clone()) }.into()
	}
}

impl<'a> From<EmberPatch<'a>> for Ember<'a> {
	fn from(value: EmberPatch<'a>) -> Self { Self::Patch(value) }
}

impl IntoLua for EmberPatch<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let ud = lua.create_userdata(self.op.into_owned())?;
		ud.set_named_user_value("tab", self.tab)?;
		ud.into_lua(lua)
	}
}
