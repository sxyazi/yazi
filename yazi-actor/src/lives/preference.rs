use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields};
use yazi_shim::{mlua::UserDataFieldsExt, strum::IntoStr};

use super::{Lives, PtrCell};

pub(super) struct Preference {
	inner: PtrCell<yazi_core::tab::Preference>,
}

impl Deref for Preference {
	type Target = yazi_core::tab::Preference;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Preference {
	pub(super) fn make(inner: &yazi_core::tab::Preference) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into() })
	}
}

impl UserData for Preference {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		// Display
		fields.add_cached_field("name", |lua, me| lua.create_string(&me.name));
		fields.add_cached_field("linemode", |lua, me| lua.create_string(&*me.linemode));
		fields.add_field_method_get("show_hidden", |_, me| Ok(me.show_hidden));

		// Sorting
		fields.add_cached_field("sort_by", |_, me| Ok(me.sort_by.into_str()));
		fields.add_field_method_get("sort_sensitive", |_, me| Ok(me.sort_sensitive));
		fields.add_field_method_get("sort_reverse", |_, me| Ok(me.sort_reverse));
		fields.add_field_method_get("sort_dir_first", |_, me| Ok(me.sort_dir_first));
		fields.add_field_method_get("sort_translit", |_, me| Ok(me.sort_translit));
		fields.add_field_method_get("sort_fallback", |_, me| Ok(me.sort_fallback.into_str()));
	}
}
