use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{FromLua, Lua, LuaSerdeExt, UserData, UserDataFields, Value};
use serde::Deserialize;
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Mixable, plugin::Previewer};

#[derive(Clone, Debug, Deserialize)]
pub struct PreviewerArc(Arc<Previewer>);

impl Deref for PreviewerArc {
	type Target = Arc<Previewer>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for PreviewerArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<Previewer> for PreviewerArc {
	fn from(value: Previewer) -> Self { Self(value.into()) }
}

impl Mixable for PreviewerArc {}

impl FromLua for PreviewerArc {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(lua.from_value::<Previewer>(value)?.into())
	}
}

impl UserData for PreviewerArc {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));

		fields.add_cached_field("name", |lua, me| lua.create_string(&*me.name));
	}
}
