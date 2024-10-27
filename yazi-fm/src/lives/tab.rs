use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields, UserDataMethods};
use yazi_plugin::url::UrlRef;

use super::{Config, Finder, Folder, Mode, Preview, SCOPE, Selected};

pub(super) struct Tab {
	inner: *const yazi_core::tab::Tab,
}

impl Deref for Tab {
	type Target = yazi_core::tab::Tab;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Tab {
	#[inline]
	pub(super) fn make(inner: &yazi_core::tab::Tab) -> mlua::Result<AnyUserData> {
		SCOPE.create_userdata(Self { inner })
	}
}

impl UserData for Tab {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id.get()));
		fields.add_field_method_get("mode", |_, me| Mode::make(&me.mode));
		fields.add_field_method_get("conf", |_, me| Config::make(&me.conf));
		fields.add_field_method_get("current", |_, me| Folder::make(None, &me.current, me));
		fields.add_field_method_get("parent", |_, me| {
			me.parent.as_ref().map(|f| Folder::make(None, f, me)).transpose()
		});

		fields.add_field_method_get("selected", |_, me| Selected::make(&me.selected));

		fields.add_field_method_get("preview", |_, me| Preview::make(me));
		fields.add_field_method_get("finder", |_, me| me.finder.as_ref().map(Finder::make).transpose());
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("name", |lua, me, ()| {
			lua.create_string(me.current.url.name().as_encoded_bytes())
		});
		methods.add_method("history", |_, me, url: UrlRef| {
			me.history.get(&url).map(|f| Folder::make(None, f, me)).transpose()
		});
	}
}
