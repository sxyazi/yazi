use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields};

use super::SCOPE;

pub(super) struct Config {
	inner: *const yazi_core::tab::Config,
}

impl Deref for Config {
	type Target = yazi_core::tab::Config;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Config {
	#[inline]
	pub(super) fn make(inner: &yazi_core::tab::Config) -> mlua::Result<AnyUserData> {
		SCOPE.create_userdata(Self { inner })
	}
}

impl UserData for Config {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("sort_by", |_, me| Ok(me.sort_by.to_string()));
		fields.add_field_method_get("sort_sensitive", |_, me| Ok(me.sort_sensitive));
		fields.add_field_method_get("sort_reverse", |_, me| Ok(me.sort_reverse));
		fields.add_field_method_get("sort_dir_first", |_, me| Ok(me.sort_dir_first));
		fields.add_field_method_get("sort_translit", |_, me| Ok(me.sort_translit));

		fields.add_field_method_get("linemode", |_, me| Ok(me.linemode.to_owned()));
		fields.add_field_method_get("show_hidden", |_, me| Ok(me.show_hidden));
	}
}
