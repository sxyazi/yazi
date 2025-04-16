use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields, Value};
use yazi_binding::cached_field;
use yazi_config::LAYOUT;

use super::{Folder, Lives, PtrCell};

pub(super) struct Preview {
	tab: PtrCell<yazi_core::tab::Tab>,

	v_folder: Option<Value>,
}

impl Deref for Preview {
	type Target = yazi_core::tab::Preview;

	fn deref(&self) -> &Self::Target { &self.tab.preview }
}

impl Preview {
	#[inline]
	pub(super) fn make(tab: &yazi_core::tab::Tab) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { tab: tab.into(), v_folder: None })
	}
}

impl UserData for Preview {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("skip", |_, me| Ok(me.skip));
		cached_field!(fields, folder, |_, me| {
			me.tab
				.hovered_folder()
				.map(|f| {
					let limit = LAYOUT.get().preview.height as usize;
					Folder::make(Some(me.skip..f.files.len().min(me.skip + limit)), f, &me.tab)
				})
				.transpose()
		});
	}
}
