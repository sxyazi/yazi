use std::ops::{Deref, Range};

use mlua::{AnyUserData, MetaMethod, UserData, UserDataFields, UserDataMethods};

use super::{File, Filter, SCOPE};

pub(super) struct Files {
	window: Range<usize>,
	folder: *const yazi_fs::Folder,
	tab:    *const yazi_core::tab::Tab,
}

impl Deref for Files {
	type Target = yazi_fs::Files;

	fn deref(&self) -> &Self::Target { &self.folder().files }
}

impl Files {
	#[inline]
	pub(super) fn make(
		window: Range<usize>,
		folder: &yazi_fs::Folder,
		tab: &yazi_core::tab::Tab,
	) -> mlua::Result<AnyUserData> {
		SCOPE.create_userdata(Self { window, folder, tab })
	}

	#[inline]
	fn folder(&self) -> &yazi_fs::Folder { unsafe { &*self.folder } }

	#[inline]
	fn tab(&self) -> &yazi_core::tab::Tab { unsafe { &*self.tab } }
}

impl UserData for Files {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("filter", |_, me| me.filter().map(Filter::make).transpose());
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.window.end - me.window.start));

		methods.add_meta_method(MetaMethod::Index, |_, me, mut idx: usize| {
			idx += me.window.start;
			if idx > me.window.end || idx == 0 {
				Ok(None)
			} else {
				Some(File::make(idx - 1, me.folder(), me.tab())).transpose()
			}
		});
	}
}
