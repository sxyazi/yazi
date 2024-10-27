use std::ops::Deref;

use mlua::{AnyUserData, LuaSerdeExt, UserData, UserDataFields};

use super::SCOPE;

pub(super) struct Tasks {
	inner: *const yazi_core::tasks::Tasks,
}

impl Deref for Tasks {
	type Target = yazi_core::tasks::Tasks;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Tasks {
	#[inline]
	pub(super) fn make(inner: &yazi_core::tasks::Tasks) -> mlua::Result<AnyUserData> {
		SCOPE.create_userdata(Self { inner })
	}
}

impl UserData for Tasks {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("progress", |lua, me| lua.to_value(&me.progress))
	}
}
