use std::ops::Deref;

use mlua::{AnyUserData, Lua, UserDataFields};

use super::SCOPE;

pub(super) struct Filter {
	inner: *const yazi_core::folder::Filter,
}

impl Deref for Filter {
	type Target = yazi_core::folder::Filter;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Filter {
	#[inline]
	pub(super) fn make(inner: &yazi_core::folder::Filter) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { inner })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_field_method_get("to_string", |_, me| Ok(me.to_string()));
		})
	}
}
