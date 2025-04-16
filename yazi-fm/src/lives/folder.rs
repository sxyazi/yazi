use std::ops::{Deref, Range};

use mlua::{AnyUserData, UserData, UserDataFields, Value};
use yazi_binding::{FolderStage, Url, cached_field};
use yazi_config::LAYOUT;

use super::{File, Files, Lives, PtrCell};

pub(super) struct Folder {
	window: Range<usize>,
	inner:  PtrCell<yazi_core::tab::Folder>,
	tab:    PtrCell<yazi_core::tab::Tab>,

	v_cwd:     Option<Value>,
	v_files:   Option<Value>,
	v_stage:   Option<Value>,
	v_window:  Option<Value>,
	v_hovered: Option<Value>,
}

impl Deref for Folder {
	type Target = yazi_core::tab::Folder;

	fn deref(&self) -> &Self::Target { &self.inner }
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

		Lives::scoped_userdata(Self {
			window,
			inner: inner.into(),
			tab: tab.into(),

			v_cwd: None,
			v_files: None,
			v_stage: None,
			v_window: None,
			v_hovered: None,
		})
	}
}

impl UserData for Folder {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, cwd, |_, me| Ok(Url::new(me.url.to_owned())));
		cached_field!(fields, files, |_, me| Files::make(0..me.files.len(), me, &me.tab));
		cached_field!(fields, stage, |_, me| Ok(FolderStage::new(me.stage)));
		cached_field!(fields, window, |_, me| Files::make(me.window.clone(), me, &me.tab));

		fields.add_field_method_get("offset", |_, me| Ok(me.offset));
		fields.add_field_method_get("cursor", |_, me| Ok(me.cursor));
		cached_field!(fields, hovered, |_, me| {
			me.hovered().map(|_| File::make(me.cursor, me, &me.tab)).transpose()
		});
	}
}
