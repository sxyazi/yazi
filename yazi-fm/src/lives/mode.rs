use std::ops::Deref;

use mlua::{AnyUserData, Lua, MetaMethod, UserDataFields, UserDataMethods};

use super::SCOPE;

pub(super) struct Mode {
	inner: *const yazi_core::tab::Mode,
}

impl Deref for Mode {
	type Target = yazi_core::tab::Mode;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Mode {
	#[inline]
	pub(super) fn make(inner: &yazi_core::tab::Mode) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { inner })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_field_method_get("is_select", |_, me| Ok(me.is_select()));
			reg.add_field_method_get("is_unset", |_, me| Ok(me.is_unset()));
			reg.add_field_method_get("is_visual", |_, me| Ok(me.is_visual()));
			reg.add_method("pending", |_, me, (idx, state): (usize, bool)| Ok(me.pending(idx, state)));

			reg.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.to_string()));
		})
	}
}
