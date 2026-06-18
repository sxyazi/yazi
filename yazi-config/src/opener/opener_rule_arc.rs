use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{FromLua, Lua, LuaSerdeExt, UserData, UserDataFields, Value};
use serde::Deserialize;
use yazi_shim::mlua::UserDataFieldsExt;

use crate::opener::OpenerRule;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct OpenerRuleArc(Arc<OpenerRule>);

impl Deref for OpenerRuleArc {
	type Target = Arc<OpenerRule>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for OpenerRuleArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<OpenerRule> for OpenerRuleArc {
	fn from(value: OpenerRule) -> Self { Self(value.into()) }
}

impl FromLua for OpenerRuleArc {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let mut inner: OpenerRule = lua.from_value(value)?;
		inner.fill();

		Ok(inner.into())
	}
}

impl UserData for OpenerRuleArc {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));
		fields.add_cached_field("run", |lua, me| lua.create_string(&*me.run));
		fields.add_field_method_get("block", |_, me| Ok(me.block));
		fields.add_field_method_get("orphan", |_, me| Ok(me.orphan));
		fields.add_cached_field("desc", |lua, me| lua.create_string(&*me.desc));
	}
}
