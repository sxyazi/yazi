use std::ops::{Deref, Range};

use mlua::{AnyUserData, UserData, UserDataFields};
use yazi_config::LAYOUT;
use yazi_shim::mlua::UserDataFieldsExt;

use super::{Entries, File, Lives, PtrCell};

pub(super) struct Folder {
	window: Range<usize>,
	inner:  PtrCell<yazi_core::tab::Folder>,
	tab:    PtrCell<yazi_core::tab::Tab>,
}

impl Deref for Folder {
	type Target = yazi_core::tab::Folder;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Folder {
	pub(super) fn make(
		window: Option<Range<usize>>,
		inner: &yazi_core::tab::Folder,
		tab: &yazi_core::tab::Tab,
	) -> mlua::Result<AnyUserData> {
		let window = match window {
			Some(w) => w,
			None => {
				let limit = LAYOUT.get().preview.height as usize;
				inner.offset..inner.entries.len().min(inner.offset + limit)
			}
		};

		Lives::scoped_userdata(Self { window, inner: inner.into(), tab: tab.into() })
	}
}

impl UserData for Folder {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("cwd", |_, me| Ok(me.url.clone()));
		fields.add_static_field("files", |_, me| Entries::make(0..me.entries.len(), me, &me.tab));
		fields.add_cached_field("stage", |_, me| Ok(me.stage.clone()));
		fields.add_static_field("window", |_, me| Entries::make(me.window.clone(), me, &me.tab));

		fields.add_field_method_get("offset", |_, me| Ok(me.offset));
		fields.add_field_method_get("cursor", |_, me| Ok(me.cursor));
		fields.add_static_field("hovered", |_, me| {
			me.hovered().map(|_| File::make(me.cursor, me, &me.tab)).transpose()
		});
	}
}
