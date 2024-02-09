use std::ops::{Deref, Range};

use mlua::{AnyUserData, Lua, MetaMethod, UserDataFields, UserDataMethods};
use yazi_config::LAYOUT;
use yazi_plugin::{bindings::Cast, url::Url};

use super::{File, Files, SCOPE};

pub(super) struct Folder {
	inner:  *const yazi_core::folder::Folder,
	window: Range<usize>,
}

impl Deref for Folder {
	type Target = yazi_core::folder::Folder;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Folder {
	#[inline]
	pub(super) fn make(
		inner: &yazi_core::folder::Folder,
		window: Option<Range<usize>>,
	) -> mlua::Result<AnyUserData<'static>> {
		let window = match window {
			Some(w) => w,
			None => {
				let limit = LAYOUT.load().preview.height as usize;
				inner.offset..inner.files.len().min(inner.offset + limit)
			}
		};

		SCOPE.create_any_userdata(Self { inner, window })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_field_method_get("cwd", |lua, me| Url::cast(lua, me.cwd.clone()));
			reg.add_field_method_get("files", |_, me| Files::make(me, 0..me.files.len()));
			reg.add_field_method_get("stage", |lua, me| lua.create_any_userdata(me.stage));
			reg.add_field_method_get("window", |_, me| Files::make(me, me.window.clone()));

			reg.add_field_method_get("offset", |_, me| Ok(me.offset));
			reg.add_field_method_get("cursor", |_, me| Ok(me.cursor));
			reg.add_field_method_get("hovered", |_, me| {
				me.hovered().map(|_| File::make(me.cursor, me)).transpose()
			});
		})?;

		lua.register_userdata_type::<yazi_core::folder::FolderStage>(|reg| {
			reg.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
				use yazi_core::folder::FolderStage::{Failed, Loaded, Loading};
				lua.create_string(match me {
					Loading => "loading",
					Loaded => "loaded",
					Failed => "failed",
				})
			});
		})?;

		Ok(())
	}
}
