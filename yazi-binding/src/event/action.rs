use std::ops::{Deref, DerefMut};

use mlua::{FromLua, Lua, UserData, UserDataFields, Value};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::event::Cmd;

pub struct Action {
	inner: yazi_shared::event::Action,
}

impl Deref for Action {
	type Target = yazi_shared::event::Action;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for Action {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl From<Action> for yazi_shared::event::Action {
	fn from(value: Action) -> Self { value.inner }
}

impl From<Action> for yazi_shared::event::Actions {
	fn from(value: Action) -> Self { Self::from(value.inner) }
}

impl Action {
	pub fn new(inner: impl Into<yazi_shared::event::Action>) -> Self { Self { inner: inner.into() } }
}

impl FromLua for Action {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::String(s) => Self { inner: s.to_str()?.parse()? },
			Value::UserData(ud) => ud.take()?,
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Action".to_owned(),
				message: Some("expected a string or an Action".to_string()),
			})?,
		})
	}
}

impl UserData for Action {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("cmd", |_, me| Ok(Cmd::new(me.cmd.clone())));
	}
}
