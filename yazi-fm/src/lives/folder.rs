use std::ops::{Deref, Range};

use mlua::{AnyUserData, Lua, MetaMethod, UserDataFields, UserDataMethods};
use yazi_config::LAYOUT;
use yazi_plugin::{bindings::Cast, url::Url};

use super::{File, Files, SCOPE};

pub(super) struct Folder {
	window: Range<usize>,
	inner:  *const yazi_fs::Folder,
	tab:    *const yazi_core::tab::Tab,
}

impl Deref for Folder {
	type Target = yazi_fs::Folder;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Folder {
	#[inline]
	pub(super) fn make(
		window: Option<Range<usize>>,
		inner: &yazi_fs::Folder,
		tab: &yazi_core::tab::Tab,
	) -> mlua::Result<AnyUserData<'static>> {
		let window = match window {
			Some(w) => w,
			None => {
				let limit = LAYOUT.load().preview.height as usize;
				inner.offset..inner.files.len().min(inner.offset + limit)
			}
		};

		SCOPE.create_any_userdata(Self { window, inner, tab })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_field_method_get("cwd", |lua, me| Url::cast(lua, me.cwd.clone()));
			reg.add_field_method_get("files", |_, me| Files::make(0..me.files.len(), me, me.tab()));
			reg.add_field_method_get("stage", |lua, me| lua.create_any_userdata(me.stage));
			reg.add_field_method_get("window", |_, me| Files::make(me.window.clone(), me, me.tab()));

			reg.add_field_method_get("offset", |_, me| Ok(me.offset));
			reg.add_field_method_get("cursor", |_, me| Ok(me.cursor));
			reg.add_field_method_get("hovered", |_, me| {
				me.hovered().map(|_| File::make(me.cursor, me, me.tab())).transpose()
			});
		})?;

		lua.register_userdata_type::<yazi_fs::FolderStage>(|reg| {
			reg.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
				use yazi_fs::FolderStage::{Failed, Loaded, Loading};
				lua.create_string(match me {
					Loading => "loading",
					Loaded => "loaded",
					Failed(_) => "failed",
				})
			});
		})?;

		Ok(())
	}

	#[inline]
	fn tab(&self) -> &yazi_core::tab::Tab { unsafe { &*self.tab } }
}
