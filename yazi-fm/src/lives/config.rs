use std::ops::Deref;

use mlua::{AnyUserData, Lua, UserDataFields};

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
	pub(super) fn make(inner: &yazi_core::tab::Config) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { inner })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_field_method_get("sort_by", |_, me| Ok(me.sort_by.to_string()));
			reg.add_field_method_get("sort_sensitive", |_, me| Ok(me.sort_sensitive));
			reg.add_field_method_get("sort_reverse", |_, me| Ok(me.sort_reverse));
			reg.add_field_method_get("sort_dir_first", |_, me| Ok(me.sort_dir_first));
			reg.add_field_method_get("sort_translit", |_, me| Ok(me.sort_translit));

			reg.add_field_method_get("linemode", |_, me| Ok(me.linemode.to_owned()));
			reg.add_field_method_get("show_hidden", |_, me| Ok(me.show_hidden));
		})
	}
}
