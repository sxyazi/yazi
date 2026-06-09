use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields};
use yazi_config::LAYOUT;
use yazi_shim::mlua::UserDataFieldsExt;

use super::{Folder, Lives, PtrCell};

pub(super) struct Preview {
	tab: PtrCell<yazi_core::tab::Tab>,
}

impl Deref for Preview {
	type Target = yazi_core::tab::Preview;

	fn deref(&self) -> &Self::Target { &self.tab.preview }
}

impl Preview {
	pub(super) fn make(tab: &yazi_core::tab::Tab) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { tab: tab.into() })
	}
}

impl UserData for Preview {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("skip", |_, me| Ok(me.skip));
		fields.add_static_field("folder", |_, me| {
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
