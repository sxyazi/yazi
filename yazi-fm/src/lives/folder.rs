use std::ops::{Deref, Range};

use mlua::{AnyUserData, IntoLuaMulti, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods};
use yazi_config::LAYOUT;
use yazi_plugin::url::Url;

use super::{File, Files, Lives};

pub(super) struct Folder {
	window: Range<usize>,
	inner:  *const yazi_core::tab::Folder,
	tab:    *const yazi_core::tab::Tab,
}

impl Deref for Folder {
	type Target = yazi_core::tab::Folder;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Folder {
	#[inline]
	pub(super) fn make(
		window: Option<Range<usize>>,
		inner: &yazi_core::tab::Folder,
		tab: &yazi_core::tab::Tab,
	) -> mlua::Result<AnyUserData> {
		let window = match window {
			Some(w) => w,
			None => {
				let limit = LAYOUT.get().preview.height as usize;
				inner.offset..inner.files.len().min(inner.offset + limit)
			}
		};

		Lives::scoped_userdata(Self { window, inner, tab })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_fs::FolderStage>(|reg| {
			use yazi_fs::FolderStage;

			reg.add_meta_method(MetaMethod::Call, |lua, me, ()| match me {
				FolderStage::Loading => false.into_lua_multi(lua),
				FolderStage::Loaded => true.into_lua_multi(lua),
				FolderStage::Failed(kind) => (true, yazi_plugin::Error::IoKind(*kind)).into_lua_multi(lua),
			});
		})?;

		Ok(())
	}

	#[inline]
	fn tab(&self) -> &yazi_core::tab::Tab { unsafe { &*self.tab } }
}

impl UserData for Folder {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("cwd", |_, me| Ok(Url(me.url.to_owned())));
		fields.add_field_method_get("files", |_, me| Files::make(0..me.files.len(), me, me.tab()));
		fields.add_field_method_get("stage", |lua, me| lua.create_any_userdata(me.stage));
		fields.add_field_method_get("window", |_, me| Files::make(me.window.clone(), me, me.tab()));

		fields.add_field_method_get("offset", |_, me| Ok(me.offset));
		fields.add_field_method_get("cursor", |_, me| Ok(me.cursor));
		fields.add_field_method_get("hovered", |_, me| {
			me.hovered().map(|_| File::make(me.cursor, me, me.tab())).transpose()
		});
	}
}
