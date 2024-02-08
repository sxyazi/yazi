use std::ops::Deref;

use mlua::{AnyUserData, Lua, UserDataFields};
use yazi_config::LAYOUT;
use yazi_plugin::{bindings::Cast, url::Url};

use super::{File, Files, SCOPE};

pub(super) struct Folder {
	inner: *const yazi_core::folder::Folder,
}

impl Deref for Folder {
	type Target = yazi_core::folder::Folder;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Folder {
	#[inline]
	pub(super) fn make(inner: &yazi_core::folder::Folder) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { inner })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_field_method_get("cwd", |lua, me| Url::cast(lua, me.cwd.clone()));
			reg.add_field_method_get("files", |_, me| Files::make(me, 0..me.files.len()));
			reg.add_field_method_get("window", |_, me| {
				Files::make(
					me,
					me.offset..me.files.len().min(me.offset + LAYOUT.load().preview.height as usize),
				)
			});

			reg.add_field_method_get("offset", |_, me| Ok(me.offset));
			reg.add_field_method_get("cursor", |_, me| Ok(me.cursor));
			reg.add_field_method_get("hovered", |_, me| {
				me.hovered().map(|_| File::make(me.cursor, me)).transpose()
			});
		})?;

		Ok(())
	}
}
