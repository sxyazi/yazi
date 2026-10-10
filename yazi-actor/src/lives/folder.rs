use std::ops::{Deref, Range};

use mlua::{AnyUserData, UserData, UserDataFields};
use yazi_config::LAYOUT;
use yazi_shared::url::UrlLike;
use yazi_shim::mlua::UserDataFieldsExt;

use super::{Entries, File, Lives, PtrCell, Tab};

pub(super) struct Folder {
	window: Range<usize>,
	inner:  PtrCell<yazi_core::tab::Folder>,
	tab:    Tab,
}

impl Deref for Folder {
	type Target = yazi_core::tab::Folder;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Folder {
	pub(super) fn make(
		offset: usize,
		inner: &yazi_core::tab::Folder,
		tab: Tab,
	) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { window: Self::window(offset, inner), inner: inner.into(), tab })
	}

	pub(super) fn window(offset: usize, inner: &yazi_core::tab::Folder) -> Range<usize> {
		let start = offset.min(inner.entries.len());
		let limit = LAYOUT.get().preview.height as usize;

		start..inner.entries.len().min(start + limit)
	}
}

impl UserData for Folder {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("cwd", |_, me| Ok(me.to_url()));
		fields.add_cached_field("file", |_, me| Ok(me.file.clone()));
		fields.add_static_field("files", |_, me| Entries::make(0..me.entries.len(), me, me.tab));
		fields.add_cached_field("stage", |_, me| Ok(me.stage.clone()));
		fields.add_static_field("window", |_, me| Entries::make(me.window.clone(), me, me.tab));

		fields.add_field_method_get("offset", |_, me| Ok(me.offset));
		fields.add_field_method_get("cursor", |_, me| Ok(me.cursor));
		fields.add_static_field("hovered", |_, me| {
			me.hovered().map(|_| File::make(me.cursor, me, me.tab)).transpose()
		});
	}
}
