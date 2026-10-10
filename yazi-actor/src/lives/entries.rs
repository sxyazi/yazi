use std::ops::{Deref, Range};

use mlua::{AnyUserData, MetaMethod, UserData, UserDataFields, UserDataMethods};
use yazi_shim::mlua::UserDataFieldsExt;

use super::{File, Filter, Lives, PtrCell, Tab};

pub(super) struct Entries {
	window: Range<usize>,
	folder: PtrCell<yazi_core::tab::Folder>,
	tab:    Tab,
}

impl Deref for Entries {
	type Target = yazi_fs::Entries;

	fn deref(&self) -> &Self::Target { &self.folder.entries }
}

impl Entries {
	pub(super) fn make(
		window: Range<usize>,
		folder: &yazi_core::tab::Folder,
		tab: Tab,
	) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { window, folder: folder.into(), tab })
	}
}

impl UserData for Entries {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_static_field("filter", |_, me| me.filter().map(Filter::make).transpose());
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.window.len()));

		methods.add_meta_method(MetaMethod::Index, |_, me, idx: usize| {
			if idx == 0 || idx > me.window.len() {
				Ok(None)
			} else {
				File::make(me.window.start + idx - 1, &me.folder, me.tab).map(Some)
			}
		});
	}
}
