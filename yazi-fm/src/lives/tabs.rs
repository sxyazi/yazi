use std::ops::Deref;

use mlua::{AnyUserData, Lua, MetaMethod, UserDataFields, UserDataMethods};

use super::{Tab, SCOPE};

pub(super) struct Tabs {
	inner: *const yazi_core::manager::Tabs,
}

impl Deref for Tabs {
	type Target = yazi_core::manager::Tabs;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Tabs {
	#[inline]
	pub(super) fn make(inner: &yazi_core::manager::Tabs) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { inner })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_field_method_get("idx", |_, me| Ok(me.cursor + 1));

			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

			reg.add_meta_method(MetaMethod::Index, |_, me, idx: usize| {
				if idx > me.len() || idx == 0 {
					Ok(None)
				} else {
					Some(Tab::make(&me[idx - 1])).transpose()
				}
			});
		})?;

		Ok(())
	}
}
