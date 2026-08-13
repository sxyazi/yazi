use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{FromLua, Lua, LuaSerdeExt, UserData, UserDataFields, Value};
use serde::Deserialize;
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Mixable, plugin::Fetcher};

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "Fetcher")]
pub struct FetcherArc {
	inner:   Arc<Fetcher>,
	pub idx: u8,
	pub rev: u16,
}

impl Deref for FetcherArc {
	type Target = Arc<Fetcher>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for FetcherArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl From<Fetcher> for FetcherArc {
	fn from(value: Fetcher) -> Self { Self { inner: value.into(), idx: 0, rev: 0 } }
}

impl Mixable for FetcherArc {}

impl FromLua for FetcherArc {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(lua.from_value::<Fetcher>(value)?.into())
	}
}

impl UserData for FetcherArc {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));

		fields.add_cached_field("name", |lua, me| lua.create_string(&*me.name));
	}
}
