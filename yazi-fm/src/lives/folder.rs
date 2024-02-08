use mlua::{AnyUserData, Lua, UserDataFields};
use yazi_config::LAYOUT;
use yazi_plugin::{bindings::Cast, url::Url};

use super::{File, Files};

pub(super) struct Folder;

impl Folder {
	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_core::folder::Folder>(|reg| {
			reg.add_field_method_get("cwd", |lua, me| Url::cast(lua, me.cwd.clone()));
			reg.add_field_method_get("files", |lua, me| {
				lua.create_any_userdata(Files::new(me, 0..me.files.len()))
			});
			reg.add_field_method_get("window", |lua, me| {
				lua.create_any_userdata(Files::new(
					me,
					me.offset..me.files.len().min(me.offset + LAYOUT.load().preview.height as usize),
				))
			});

			reg.add_field_method_get("offset", |_, me| Ok(me.offset));
			reg.add_field_method_get("cursor", |_, me| Ok(me.cursor));
			reg.add_field_method_get("hovered", |lua, me| {
				me.hovered().map(|_| lua.create_any_userdata(File::new(me.cursor, me))).transpose()
			});
		})?;

		Ok(())
	}
}

impl Folder {
	#[inline]
	pub(crate) fn make<'a>(
		scope: &mlua::Scope<'a, 'a>,
		inner: &'a yazi_core::folder::Folder,
	) -> mlua::Result<AnyUserData<'a>> {
		scope.create_any_userdata_ref(inner)
	}
}
