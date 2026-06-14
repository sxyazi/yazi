use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields};
use yazi_shim::mlua::UserDataFieldsExt;

use super::{Lives, PtrCell};
use crate::lives::InputAlt;

pub(super) struct Input {
	inner: PtrCell<yazi_core::input::Input>,
}

impl Deref for Input {
	type Target = yazi_core::input::Input;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Input {
	pub(super) fn make(inner: &yazi_core::input::Input) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into() })
	}
}

impl UserData for Input {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("alt", |_, me| me.alt.as_ref().map(InputAlt::make).transpose());
	}
}
