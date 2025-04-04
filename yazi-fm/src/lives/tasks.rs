use std::ops::Deref;

use mlua::{AnyUserData, Lua, LuaSerdeExt, UserData, UserDataFields, Value};
use yazi_binding::cached_field;

use super::Lives;

pub(super) struct Tasks {
	inner: *const yazi_core::tasks::Tasks,

	v_progress: Option<Value>,
}

impl Deref for Tasks {
	type Target = yazi_core::tasks::Tasks;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Tasks {
	#[inline]
	pub(super) fn make(inner: &yazi_core::tasks::Tasks) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner, v_progress: None })
	}
}

impl UserData for Tasks {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, progress, |lua: &Lua, me: &Self| lua.to_value(&me.progress));
	}
}
