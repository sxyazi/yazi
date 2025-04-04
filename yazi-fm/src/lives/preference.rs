use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields, Value};
use yazi_binding::cached_field;

use super::Lives;

pub(super) struct Preference {
	inner: *const yazi_core::tab::Preference,

	v_sort_by:  Option<Value>,
	v_linemode: Option<Value>,
}

impl Deref for Preference {
	type Target = yazi_core::tab::Preference;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Preference {
	#[inline]
	pub(super) fn make(inner: &yazi_core::tab::Preference) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner, v_sort_by: None, v_linemode: None })
	}
}

impl UserData for Preference {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, sort_by, |_, me: &Self| Ok(me.sort_by.to_string()));
		fields.add_field_method_get("sort_sensitive", |_, me| Ok(me.sort_sensitive));
		fields.add_field_method_get("sort_reverse", |_, me| Ok(me.sort_reverse));
		fields.add_field_method_get("sort_dir_first", |_, me| Ok(me.sort_dir_first));
		fields.add_field_method_get("sort_translit", |_, me| Ok(me.sort_translit));

		cached_field!(fields, linemode, |_, me: &Self| Ok(me.linemode.to_string()));
		fields.add_field_method_get("show_hidden", |_, me| Ok(me.show_hidden));
	}
}
