use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields, UserDataMethods, Value};
use yazi_binding::{Id, UrlRef, cached_field};

use super::{Finder, Folder, Lives, Mode, Preference, Preview, PtrCell, Selected};

pub(super) struct Tab {
	inner: PtrCell<yazi_core::tab::Tab>,

	v_name:     Option<Value>,
	v_mode:     Option<Value>,
	v_pref:     Option<Value>,
	v_current:  Option<Value>,
	v_parent:   Option<Value>,
	v_selected: Option<Value>,
	v_preview:  Option<Value>,
	v_finder:   Option<Value>,
}

impl Deref for Tab {
	type Target = yazi_core::tab::Tab;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Tab {
	#[inline]
	pub(super) fn make(inner: &yazi_core::tab::Tab) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self {
			inner: inner.into(),

			v_name:     None,
			v_mode:     None,
			v_pref:     None,
			v_current:  None,
			v_parent:   None,
			v_selected: None,
			v_preview:  None,
			v_finder:   None,
		})
	}
}

impl UserData for Tab {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(Id(me.id)));
		cached_field!(fields, name, |lua, me| {
			lua.create_string(me.current.url.name().as_encoded_bytes())
		});

		cached_field!(fields, mode, |_, me| Mode::make(&me.mode));
		cached_field!(fields, pref, |_, me| Preference::make(&me.pref));
		cached_field!(fields, current, |_, me| Folder::make(None, &me.current, me));
		cached_field!(fields, parent, |_, me| {
			me.parent.as_ref().map(|f| Folder::make(None, f, me)).transpose()
		});

		cached_field!(fields, selected, |_, me| Selected::make(&me.selected));

		cached_field!(fields, preview, |_, me| Preview::make(me));
		cached_field!(fields, finder, |_, me| me.finder.as_ref().map(Finder::make).transpose());
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("history", |_, me, url: UrlRef| {
			me.history.get(&url).map(|f| Folder::make(None, f, me)).transpose()
		});
	}
}
