use std::{borrow::Cow, ops::Deref};

use mlua::{AnyUserData, UserData, UserDataFields, UserDataMethods};
use yazi_shared::url::UrlRef;
use yazi_shim::mlua::UserDataFieldsExt;

use super::{Finder, Folder, Lives, Mode, Preference, Preview, PtrCell, Selected};

pub(super) struct Tab {
	inner: PtrCell<yazi_core::tab::Tab>,
}

impl Deref for Tab {
	type Target = yazi_core::tab::Tab;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Tab {
	pub(super) fn make(inner: &yazi_core::tab::Tab) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into() })
	}
}

impl UserData for Tab {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));
		fields.add_cached_field("name", |lua, me| match me.name() {
			Cow::Borrowed(s) => lua.create_string(s),
			Cow::Owned(s) => lua.create_external_string(s),
		});

		fields.add_static_field("mode", |_, me| Mode::make(&me.mode));
		fields.add_static_field("pref", |_, me| Preference::make(&me.pref));
		fields.add_static_field("current", |_, me| Folder::make(None, &me.current, me));
		fields.add_static_field("parent", |_, me| {
			me.parent.as_ref().map(|f| Folder::make(None, f, me)).transpose()
		});

		fields.add_static_field("selected", |_, me| Selected::make(&me.selected));

		fields.add_static_field("preview", |_, me| Preview::make(me));
		fields.add_static_field("finder", |_, me| me.finder.as_ref().map(Finder::make).transpose());
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("history", |_, me, url: UrlRef| {
			me.history.get(url.as_ref()).map(|f| Folder::make(None, f, me)).transpose()
		});
	}
}
