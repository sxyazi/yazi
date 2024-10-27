use std::ops::Deref;

use mlua::{AnyUserData, MetaMethod, UserData, UserDataMethods};

use super::SCOPE;

pub(super) struct Filter {
	inner: *const yazi_fs::Filter,
}

impl Deref for Filter {
	type Target = yazi_fs::Filter;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Filter {
	#[inline]
	pub(super) fn make(inner: &yazi_fs::Filter) -> mlua::Result<AnyUserData> {
		SCOPE.create_userdata(Self { inner })
	}
}

impl UserData for Filter {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.to_string()));
	}
}
