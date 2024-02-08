use std::ops::Deref;

use mlua::{AnyUserData, Lua, UserDataFields};

use super::{Folder, SCOPE};

pub(super) struct Preview {
	tab: *const yazi_core::tab::Tab,
}

impl Deref for Preview {
	type Target = yazi_core::tab::Preview;

	fn deref(&self) -> &Self::Target { &self.tab().preview }
}

impl Preview {
	#[inline]
	pub(super) fn make(tab: &yazi_core::tab::Tab) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { tab })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_field_method_get("skip", |_, me| Ok(me.skip));
			reg.add_field_method_get("folder", |_, me| {
				me.tab()
					.current
					.hovered()
					.filter(|&f| f.is_dir())
					.and_then(|f| me.tab().history(&f.url))
					.map(Folder::make)
					.transpose()
			});
		})
	}

	#[inline]
	fn tab(&self) -> &yazi_core::tab::Tab { unsafe { &*self.tab } }
}
