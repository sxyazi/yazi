use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields};

use super::{Lives, PtrCell};

pub(super) struct Which {
	inner: PtrCell<yazi_core::which::Which>,
}

impl Deref for Which {
	type Target = yazi_core::which::Which;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Which {
	pub(super) fn make(inner: &yazi_core::which::Which) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into() })
	}
}

impl UserData for Which {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {}
}
