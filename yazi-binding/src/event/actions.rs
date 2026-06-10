use std::ops::{Deref, DerefMut};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};

use crate::event::Action;

#[derive(Clone)]
pub struct Actions {
	inner: yazi_shared::event::Actions,
}

impl Deref for Actions {
	type Target = yazi_shared::event::Actions;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for Actions {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl From<Actions> for yazi_shared::event::Actions {
	fn from(value: Actions) -> Self { value.inner }
}

impl Actions {
	pub fn new(inner: yazi_shared::event::Actions) -> Self { Self { inner } }
}

impl FromLua for Actions {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let inner = match value {
			Value::Table(t) => t
				.sequence_values::<Action>()
				.map(|a| a.map(Into::into))
				.collect::<mlua::Result<Vec<_>>>()?
				.into(),
			v @ Value::String(_) => Action::from_lua(v, lua)?.into(),
			_ => Err("expected a string or a table of actions".into_lua_err())?,
		};

		Ok(Self { inner })
	}
}

impl IntoLua for Actions {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_sequence_from(self.inner.into_iter().map(Action::new))?.into_lua(lua)
	}
}
