use std::{collections::{btree_set, BTreeSet}, ops::Deref};

use mlua::{AnyUserData, Lua, MetaMethod, UserDataMethods, UserDataRefMut};
use yazi_plugin::{bindings::Cast, url::Url};

use super::SCOPE;

#[derive(Clone, Copy)]
pub(super) struct Selected {
	inner: *const BTreeSet<yazi_shared::fs::Url>,
}

impl Deref for Selected {
	type Target = BTreeSet<yazi_shared::fs::Url>;

	fn deref(&self) -> &Self::Target { self.inner() }
}

impl Selected {
	#[inline]
	pub(crate) fn make(inner: &BTreeSet<yazi_shared::fs::Url>) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { inner })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

			reg.add_meta_method(MetaMethod::Pairs, |lua, me, ()| {
				let iter = lua.create_function(|lua, mut iter: UserDataRefMut<SelectedIter>| {
					Ok(if let Some(url) = iter.0.next() { Some(Url::cast(lua, url.clone())?) } else { None })
				})?;

				Ok((iter, SelectedIter::make(me.inner())))
			});
		})?;

		Ok(())
	}

	#[inline]
	fn inner(&self) -> &'static BTreeSet<yazi_shared::fs::Url> { unsafe { &*self.inner } }
}

struct SelectedIter(btree_set::Iter<'static, yazi_shared::fs::Url>);

impl SelectedIter {
	#[inline]
	fn make(selected: &BTreeSet<yazi_shared::fs::Url>) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self(selected.iter()))
	}
}
