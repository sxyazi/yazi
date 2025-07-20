use std::ops::{Deref, Range};

use mlua::{AnyUserData, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use yazi_binding::cached_field;

use super::{File, Filter, Lives, PtrCell};

pub(super) struct Files {
	window: Range<usize>,
	folder: PtrCell<yazi_core::tab::Folder>,
	tab:    PtrCell<yazi_core::tab::Tab>,

	v_filter: Option<Value>,
}

impl Deref for Files {
	type Target = yazi_fs::Files;

	fn deref(&self) -> &Self::Target { &self.folder.files }
}

impl Files {
	#[inline]
	pub(super) fn make(
		window: Range<usize>,
		folder: &yazi_core::tab::Folder,
		tab: &yazi_core::tab::Tab,
	) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { window, folder: folder.into(), tab: tab.into(), v_filter: None })
	}
}

impl UserData for Files {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, filter, |_, me| me.filter().map(Filter::make).transpose());
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.window.end - me.window.start));

		methods.add_meta_method(MetaMethod::Index, |_, me, mut idx: usize| {
			idx += me.window.start;
			if idx > me.window.end || idx == 0 {
				Ok(None)
			} else {
				Some(File::make(idx - 1, &me.folder, &me.tab)).transpose()
			}
		});
	}
}
