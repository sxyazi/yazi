use std::ops::Deref;

use mlua::UserData;

pub struct Action {
	inner: yazi_shared::event::Action,
}

impl Deref for Action {
	type Target = yazi_shared::event::Action;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Action {
	pub fn new(inner: impl Into<yazi_shared::event::Action>) -> Self { Self { inner: inner.into() } }
}

impl UserData for Action {}
