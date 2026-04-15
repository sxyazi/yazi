use std::{ops::Deref, sync::Arc};

use mlua::{AnyUserData, ExternalError, FromLua, Lua, UserData, UserDataFields, Value};
use yazi_config::plugin::previewer_id;

use crate::{Id, cached_field};

const EXPECTED: &str = "expected a table or Previewer";

#[derive(Clone)]
pub struct Previewer {
	inner: Arc<yazi_config::plugin::Previewer>,

	v_name: Option<Value>,
}

impl Deref for Previewer {
	type Target = yazi_config::plugin::Previewer;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<Previewer> for Arc<yazi_config::plugin::Previewer> {
	fn from(value: Previewer) -> Self { value.inner }
}

impl Previewer {
	pub fn new(inner: impl Into<Arc<yazi_config::plugin::Previewer>>) -> Self {
		Self { inner: inner.into(), v_name: None }
	}
}

impl TryFrom<mlua::Table> for Previewer {
	type Error = mlua::Error;

	fn try_from(t: mlua::Table) -> Result<Self, Self::Error> {
		Ok(Self::new(yazi_config::plugin::Previewer {
			id:   previewer_id(),
			url:  t.raw_get::<Option<mlua::String>>("url")?.map(|s| s.to_str()?.parse()).transpose()?,
			mime: t.raw_get::<Option<mlua::String>>("mime")?.map(|s| s.to_str()?.parse()).transpose()?,
			run:  t.raw_get::<mlua::String>("run")?.to_str()?.parse()?,
		}))
	}
}

impl TryFrom<AnyUserData> for Previewer {
	type Error = mlua::Error;

	fn try_from(value: AnyUserData) -> Result<Self, Self::Error> {
		Ok(Self::new(value.borrow::<Self>()?.clone()))
	}
}

impl FromLua for Previewer {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(tbl) => Self::try_from(tbl),
			Value::UserData(ud) => Self::try_from(ud),
			_ => Err(EXPECTED.into_lua_err())?,
		}
	}
}

impl UserData for Previewer {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(Id(me.id)));

		cached_field!(fields, name, |lua, me| lua.create_string(&*me.name));
	}
}
