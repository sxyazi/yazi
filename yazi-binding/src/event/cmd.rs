use std::{mem, ops::{Deref, DerefMut}};

use mlua::{UserData, UserDataFields};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::Sendable;

pub struct Cmd {
	inner: yazi_shared::event::Cmd,
}

impl Deref for Cmd {
	type Target = yazi_shared::event::Cmd;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for Cmd {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl Cmd {
	pub fn new(inner: impl Into<yazi_shared::event::Cmd>) -> Self { Self { inner: inner.into() } }
}

impl UserData for Cmd {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field_mut("name", |_, me| Ok(mem::take(&mut me.name)));

		fields.add_cached_field_mut("args", |lua, me| {
			Sendable::args_to_table(lua, mem::take(&mut me.args))
		});
	}
}
