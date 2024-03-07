use std::{collections::hash_set, ops::Deref};

use mlua::{AnyUserData, IntoLuaMulti, Lua, MetaMethod, UserDataFields, UserDataMethods, UserDataRefMut};
use yazi_plugin::{bindings::Cast, url::Url};

use super::{Iter, SCOPE};

pub(super) struct Yanked {
	inner: *const yazi_core::manager::Yanked,
}

impl Deref for Yanked {
	type Target = yazi_core::manager::Yanked;

	fn deref(&self) -> &Self::Target { self.inner() }
}

impl Yanked {
	#[inline]
	pub(super) fn make(inner: &yazi_core::manager::Yanked) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { inner })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_field_method_get("is_cut", |_, me| Ok(me.cut));

			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

			reg.add_meta_method(MetaMethod::Pairs, |lua, me, ()| {
				let iter = lua.create_function(
					|lua, mut iter: UserDataRefMut<Iter<hash_set::Iter<yazi_shared::fs::Url>, _>>| {
						if let Some(next) = iter.next() {
							(next.0, Url::cast(lua, next.1.clone())?).into_lua_multi(lua)
						} else {
							().into_lua_multi(lua)
						}
					},
				)?;

				Ok((iter, Iter::make(me.inner().iter())))
			});
		})
	}

	#[inline]
	fn inner(&self) -> &'static yazi_core::manager::Yanked { unsafe { &*self.inner } }
}
