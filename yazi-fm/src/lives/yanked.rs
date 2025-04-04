use std::{collections::hash_set, ops::Deref};

use mlua::{AnyUserData, IntoLuaMulti, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRefMut};
use yazi_binding::Url;

use super::{Iter, Lives, PtrCell};

pub(super) struct Yanked {
	inner: PtrCell<yazi_core::mgr::Yanked>,
}

impl Deref for Yanked {
	type Target = yazi_core::mgr::Yanked;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Yanked {
	#[inline]
	pub(super) fn make(inner: &yazi_core::mgr::Yanked) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into() })
	}
}

impl UserData for Yanked {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("is_cut", |_, me| Ok(me.cut));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

		methods.add_meta_method(MetaMethod::Pairs, |lua, me, ()| {
			let iter = lua.create_function(
				|lua, mut iter: UserDataRefMut<Iter<hash_set::Iter<yazi_shared::url::Url>, _>>| {
					if let Some(next) = iter.next() {
						(next.0, Url::new(next.1.clone())).into_lua_multi(lua)
					} else {
						().into_lua_multi(lua)
					}
				},
			)?;

			Ok((iter, Iter::make(me.inner.as_static().iter())))
		});
	}
}
