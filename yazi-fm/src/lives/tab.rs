use std::ops::Deref;

use mlua::{AnyUserData, Lua, UserData, UserDataFields, UserDataMethods, Value};
use yazi_binding::{UrlRef, cached_field};
use yazi_plugin::Id;

use super::{Finder, Folder, Lives, Mode, Preference, Preview, Selected};

pub(super) struct Tab {
	inner: *const yazi_core::tab::Tab,

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

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Tab {
	#[inline]
	pub(super) fn make(inner: &yazi_core::tab::Tab) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self {
			inner,

			v_name: None,
			v_mode: None,
			v_pref: None,
			v_current: None,
			v_parent: None,
			v_selected: None,
			v_preview: None,
			v_finder: None,
		})
	}
}

impl UserData for Tab {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(Id(me.id)));
		cached_field!(fields, name, |lua: &Lua, me: &Self| {
			lua.create_string(me.current.url.name().as_encoded_bytes())
		});

		cached_field!(fields, mode, |_, me: &Self| Mode::make(&me.mode));
		cached_field!(fields, pref, |_, me: &Self| Preference::make(&me.pref));
		cached_field!(fields, current, |_, me: &Self| Folder::make(None, &me.current, me));
		cached_field!(fields, parent, |_, me: &Self| {
			me.parent.as_ref().map(|f| Folder::make(None, f, me)).transpose()
		});

		cached_field!(fields, selected, |_, me: &Self| Selected::make(&me.selected));

		cached_field!(fields, preview, |_, me: &Self| Preview::make(me));
		cached_field!(fields, finder, |_, me: &Self| me.finder.as_ref().map(Finder::make).transpose());
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("history", |_, me, url: UrlRef| {
			me.history.get(&url).map(|f| Folder::make(None, f, me)).transpose()
		});
	}
}
