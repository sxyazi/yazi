use std::{mem, ops::{Deref, DerefMut}};

use mlua::{UserData, UserDataFields};
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

impl Action {
	pub fn new(inner: impl Into<yazi_shared::event::Action>) -> Self { Self { inner: inner.into() } }
}

impl UserData for Action {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field_mut("cmd", |_, me| Ok(Cmd::new(mem::take(&mut me.cmd))));
	}
}
