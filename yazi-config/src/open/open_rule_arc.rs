use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{FromLua, Lua, LuaSerdeExt, UserData, UserDataFields, Value};
use serde::Deserialize;
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Mixable, open::OpenRule};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct OpenRuleArc(Arc<OpenRule>);

impl Deref for OpenRuleArc {
	type Target = Arc<OpenRule>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for OpenRuleArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<OpenRule> for OpenRuleArc {
	fn from(value: OpenRule) -> Self { Self(value.into()) }
}

impl Mixable for OpenRuleArc {
	fn any_file(&self) -> bool { self.0.any_file() }

	fn any_dir(&self) -> bool { self.0.any_dir() }
}

impl FromLua for OpenRuleArc {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl UserData for OpenRuleArc {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));
		fields.add_cached_field("use", |lua, me| {
			lua.create_sequence_from(me.r#use.iter().map(|s| s.as_str()))
		});
	}
}
