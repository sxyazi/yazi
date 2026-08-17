use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{FromLua, Lua, LuaSerdeExt, UserData, UserDataFields, Value};
use serde::Deserialize;
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Mixable, plugin::Spotter};

#[derive(Clone, Debug, Deserialize)]
pub struct SpotterArc(Arc<Spotter>);

impl Deref for SpotterArc {
	type Target = Arc<Spotter>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for SpotterArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<Spotter> for SpotterArc {
	fn from(value: Spotter) -> Self { Self(value.into()) }
}

impl Mixable for SpotterArc {
	fn any_file(&self) -> bool { self.0.any_file() }

	fn any_dir(&self) -> bool { self.0.any_dir() }
}

impl FromLua for SpotterArc {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(lua.from_value::<Spotter>(value)?.into())
	}
}

impl UserData for SpotterArc {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));

		fields.add_cached_field("name", |lua, me| lua.create_string(&*me.name));
	}
}
