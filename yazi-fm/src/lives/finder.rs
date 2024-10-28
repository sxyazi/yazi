use std::ops::Deref;

use mlua::{AnyUserData, Lua, MetaMethod, UserDataMethods};

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
	pub(super) fn make(inner: &yazi_core::tab::Finder) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { inner })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.filter.to_string()));
		})
	}
}
