use std::ops::Deref;

use mlua::UserData;

pub struct Cmd {
	inner: yazi_shared::event::Cmd,
}

impl Deref for Cmd {
	type Target = yazi_shared::event::Cmd;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Cmd {
	pub fn new(inner: impl Into<yazi_shared::event::Cmd>) -> Self { Self { inner: inner.into() } }
}

impl UserData for Cmd {}
