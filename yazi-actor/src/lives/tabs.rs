use std::ops::Deref;

use mlua::{AnyUserData, MetaMethod, UserData, UserDataFields, UserDataMethods};

use super::{Lives, PtrCell, Tab};

pub(super) struct Tabs {
	inner: PtrCell<yazi_core::mgr::Tabs>,
}

impl Deref for Tabs {
	type Target = yazi_core::mgr::Tabs;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Tabs {
	#[inline]
	pub(super) fn make(inner: &yazi_core::mgr::Tabs) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into() })
	}
}

impl UserData for Tabs {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("idx", |_, me| Ok(me.cursor + 1));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

		methods.add_meta_method(MetaMethod::Index, |_, me, idx: usize| {
			if idx > me.len() || idx == 0 { Ok(None) } else { Some(Tab::make(&me[idx - 1])).transpose() }
		});
	}
}
