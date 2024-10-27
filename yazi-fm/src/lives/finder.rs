use std::ops::Deref;

use mlua::{AnyUserData, MetaMethod, UserData, UserDataMethods};

use super::SCOPE;

pub(super) struct Finder {
	inner: *const yazi_core::tab::Finder,
}

impl Deref for Finder {
	type Target = yazi_core::tab::Finder;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Finder {
	#[inline]
	pub(super) fn make(inner: &yazi_core::tab::Finder) -> mlua::Result<AnyUserData> {
		SCOPE.create_userdata(Self { inner })
	}
}

impl UserData for Finder {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.filter.to_string()));
	}
}
