use std::ops::Deref;

use mlua::{AnyUserData, Lua, UserDataFields, UserDataMethods};

use super::{Config, Folder, Mode, Preview, SCOPE};

pub(super) struct Tab {
	inner: *const yazi_core::tab::Tab,
}

impl Deref for Tab {
	type Target = yazi_core::tab::Tab;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Tab {
	#[inline]
	pub(super) fn make(inner: &yazi_core::tab::Tab) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { inner })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_method("name", |lua, me, ()| {
				Some(
					lua.create_string(
						me.current
							.cwd
							.file_name()
							.map_or(me.current.cwd.as_os_str().as_encoded_bytes(), |n| n.as_encoded_bytes()),
					),
				)
				.transpose()
			});

			reg.add_field_method_get("mode", |_, me| Mode::make(&me.mode));
			reg.add_field_method_get("conf", |_, me| Config::make(&me.conf));
			reg.add_field_method_get("parent", |_, me| me.parent.as_ref().map(Folder::make).transpose());
			reg.add_field_method_get("current", |_, me| Folder::make(&me.current));
			reg.add_field_method_get("preview", |_, me| Preview::make(me));
		})?;

		Ok(())
	}
}
