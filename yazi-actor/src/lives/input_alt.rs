use std::{ops::Deref, sync::Arc};

use mlua::{AnyUserData, UserData, UserDataFields};
use parking_lot::Mutex;
use yazi_shim::mlua::UserDataFieldsExt;

use super::{Lives, PtrCell};

pub(super) struct InputAlt(PtrCell<Arc<Mutex<yazi_widgets::input::Input>>>);

impl Deref for InputAlt {
	type Target = Arc<Mutex<yazi_widgets::input::Input>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl InputAlt {
	pub(super) fn make(inner: &Arc<Mutex<yazi_widgets::input::Input>>) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self(inner.into()))
	}
}

impl UserData for InputAlt {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("value", |lua, me| lua.create_string(me.lock().value()));
	}
}
