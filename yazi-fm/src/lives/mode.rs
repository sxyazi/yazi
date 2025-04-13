use std::ops::Deref;

use mlua::{AnyUserData, MetaMethod, UserData, UserDataFields, UserDataMethods};

use super::{Lives, PtrCell};

pub(super) struct Mode {
	inner: PtrCell<yazi_core::tab::Mode>,
}

impl Deref for Mode {
	type Target = yazi_core::tab::Mode;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Mode {
	#[inline]
	pub(super) fn make(inner: &yazi_core::tab::Mode) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into() })
	}
}

impl UserData for Mode {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("is_select", |_, me| Ok(me.is_select()));
		fields.add_field_method_get("is_unset", |_, me| Ok(me.is_unset()));
		fields.add_field_method_get("is_visual", |_, me| Ok(me.is_visual()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.to_string()));
	}
}
