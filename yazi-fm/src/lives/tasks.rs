use std::ops::Deref;

use mlua::{AnyUserData, Lua, LuaSerdeExt, UserDataFields};

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
	pub(crate) fn make(inner: &yazi_core::tasks::Tasks) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { inner })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_field_method_get("progress", |lua, me| lua.to_value(&me.progress))
		})?;

		Ok(())
	}
}
