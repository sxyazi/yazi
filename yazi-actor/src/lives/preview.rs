use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields};
use yazi_shim::mlua::UserDataFieldsExt;

use super::{Folder, Lives, Tab};

pub(super) struct Preview {
	tab: Tab,
}

impl Deref for Preview {
	type Target = yazi_core::tab::Preview;

	fn deref(&self) -> &Self::Target { &self.tab.preview }
}

impl Preview {
	pub(super) fn make(tab: Tab) -> mlua::Result<AnyUserData> { Lives::scoped_userdata(Self { tab }) }
}

impl UserData for Preview {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("skip", |_, me| Ok(me.skip));
		fields.add_static_field("folder", |_, me| {
			me.tab.hovered_folder().map(|f| Folder::make(me.skip, f, me.tab)).transpose()
		});
	}
}
