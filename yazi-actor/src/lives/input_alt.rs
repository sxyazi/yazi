use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields};
use yazi_shim::mlua::UserDataFieldsExt;

use super::{Lives, PtrCell};

pub(super) struct InputAlt(PtrCell<yazi_core::input::InputAlt>);

impl Deref for InputAlt {
	type Target = yazi_core::input::InputAlt;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl InputAlt {
	pub(super) fn make(inner: &yazi_core::input::InputAlt) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self(inner.into()))
	}
}

impl UserData for InputAlt {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("value", |lua, me| lua.create_string(me.lock().value()));
	}
}
