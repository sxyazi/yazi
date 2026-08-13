use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{FromLua, Lua, LuaSerdeExt, UserData, UserDataFields, Value};
use serde::Deserialize;
use yazi_shared::data::Sendable;
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Mixable, plugin::Preloader};

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "Preloader")]
pub struct PreloaderArc {
	inner:   Arc<Preloader>,
	pub idx: u8,
	pub rev: u16,
}

impl Deref for PreloaderArc {
	type Target = Arc<Preloader>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for PreloaderArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl From<Preloader> for PreloaderArc {
	fn from(value: Preloader) -> Self { Self { inner: value.into(), idx: 0, rev: 0 } }
}

impl Mixable for PreloaderArc {}

impl FromLua for PreloaderArc {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(lua.from_value::<Preloader>(value)?.into())
	}
}

impl UserData for PreloaderArc {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));

		fields.add_cached_field("name", |lua, me| lua.create_string(&*me.name));
		fields.add_cached_field("args", |lua, me| Sendable::args_to_table_ref(lua, &me.args));
	}
}
