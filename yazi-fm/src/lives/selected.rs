use std::{collections::{HashMap, hash_map}, ops::Deref};

use mlua::{AnyUserData, IntoLuaMulti, MetaMethod, UserData, UserDataMethods, UserDataRefMut};
use yazi_plugin::{bindings::Cast, url::Url};

use super::{Iter, SCOPE};

#[derive(Clone, Copy)]
pub(super) struct Selected {
	inner: *const HashMap<yazi_shared::fs::Url, u64>,
}

impl Deref for Selected {
	type Target = HashMap<yazi_shared::fs::Url, u64>;

	fn deref(&self) -> &Self::Target { self.inner() }
}

impl Selected {
	#[inline]
	pub(super) fn make(inner: &HashMap<yazi_shared::fs::Url, u64>) -> mlua::Result<AnyUserData> {
		SCOPE.create_userdata(Self { inner })
	}

	#[inline]
	fn inner(&self) -> &'static HashMap<yazi_shared::fs::Url, u64> { unsafe { &*self.inner } }
}

impl UserData for Selected {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

		methods.add_meta_method(MetaMethod::Pairs, |lua, me, ()| {
			let iter = lua.create_function(
				// FIXME: UserDataRef
				|lua, mut iter: UserDataRefMut<Iter<hash_map::Keys<yazi_shared::fs::Url, u64>, _>>| {
					if let Some(next) = iter.next() {
						(next.0, Url::cast(lua, next.1.clone())?).into_lua_multi(lua)
					} else {
						().into_lua_multi(lua)
					}
				},
			)?;

			Ok((iter, Iter::make(me.inner().keys())))
		});
	}
}
