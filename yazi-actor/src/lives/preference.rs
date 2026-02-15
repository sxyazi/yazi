use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields, Value};
use yazi_binding::cached_field;

use super::{Lives, PtrCell};

pub(super) struct Preference {
	inner: PtrCell<yazi_core::tab::Preference>,

	v_name:     Option<Value>,
	v_linemode: Option<Value>,

	v_sort_by: Option<Value>,
}

impl Deref for Preference {
	type Target = yazi_core::tab::Preference;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Preference {
	pub(super) fn make(inner: &yazi_core::tab::Preference) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self {
			inner: inner.into(),

			v_name:     None,
			v_linemode: None,

			v_sort_by: None,
		})
	}
}

impl UserData for Preference {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		// Display
		cached_field!(fields, name, |lua, me| lua.create_string(&me.name));
		cached_field!(fields, linemode, |lua, me| lua.create_string(&me.linemode));
		fields.add_field_method_get("show_hidden", |_, me| Ok(me.show_hidden));

		// Sorting
		cached_field!(fields, sort_by, |_, me| Ok(me.sort_by.to_string()));
		fields.add_field_method_get("sort_sensitive", |_, me| Ok(me.sort_sensitive));
		fields.add_field_method_get("sort_reverse", |_, me| Ok(me.sort_reverse));
		fields.add_field_method_get("sort_dir_first", |_, me| Ok(me.sort_dir_first));
		fields.add_field_method_get("sort_translit", |_, me| Ok(me.sort_translit));
		fields.add_field_method_get("sort_fallback", |_, me| Ok(me.sort_fallback.to_string()));
	}
}
