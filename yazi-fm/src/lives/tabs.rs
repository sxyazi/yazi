use std::ops::Deref;

use mlua::{AnyUserData, MetaMethod, UserData, UserDataFields, UserDataMethods};

use super::{SCOPE, Tab};

pub(super) struct Tabs {
	inner: *const yazi_core::manager::Tabs,
}

impl Deref for Tabs {
	type Target = yazi_core::manager::Tabs;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Tabs {
	#[inline]
	pub(super) fn make(inner: &yazi_core::manager::Tabs) -> mlua::Result<AnyUserData> {
		SCOPE.create_userdata(Self { inner })
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
