use std::ops::Deref;

use indexmap::{IndexMap, map::Keys};
use mlua::{AnyUserData, IntoLuaMulti, MetaMethod, UserData, UserDataMethods, UserDataRefMut};
use yazi_binding::Url;

use super::{Iter, Lives, PtrCell};

#[derive(Clone, Copy)]
pub(super) struct Selected {
	inner: PtrCell<IndexMap<yazi_shared::url::Url, u64>>,
}

impl Deref for Selected {
	type Target = IndexMap<yazi_shared::url::Url, u64>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Selected {
	#[inline]
	pub(super) fn make(inner: &IndexMap<yazi_shared::url::Url, u64>) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into() })
	}
}

impl UserData for Selected {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

		methods.add_meta_method(MetaMethod::Pairs, |lua, me, ()| {
			let iter = lua.create_function(
				|lua, mut iter: UserDataRefMut<Iter<Keys<yazi_shared::url::Url, u64>, _>>| {
					if let Some(next) = iter.next() {
						(next.0, Url::new(next.1.clone())).into_lua_multi(lua)
					} else {
						().into_lua_multi(lua)
					}
				},
			)?;

			Ok((iter, Iter::make(me.inner.as_static().keys())))
		});
	}
}
