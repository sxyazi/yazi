use std::{collections::{hash_set, HashSet}, ops::Deref};

use mlua::{AnyUserData, IntoLuaMulti, Lua, MetaMethod, UserDataMethods, UserDataRefMut};
use yazi_plugin::{bindings::Cast, url::Url};

use super::{Iter, SCOPE};

#[derive(Clone, Copy)]
pub(super) struct Selected {
	inner: *const HashSet<yazi_shared::fs::Url>,
}

impl Deref for Selected {
	type Target = HashSet<yazi_shared::fs::Url>;

	fn deref(&self) -> &Self::Target { self.inner() }
}

impl Selected {
	#[inline]
	pub(super) fn make(inner: &HashSet<yazi_shared::fs::Url>) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { inner })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
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
		})?;

		Ok(())
	}

	#[inline]
	fn inner(&self) -> &'static HashSet<yazi_shared::fs::Url> { unsafe { &*self.inner } }
}
