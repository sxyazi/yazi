use std::{borrow::Cow, collections::{HashMap, hash_map}};

use mlua::{AnyUserData, IntoLua, IntoLuaMulti, Lua, MetaMethod, UserData, UserDataRefMut, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyBulk<'a> {
	pub changes: HashMap<Cow<'a, Url>, Cow<'a, Url>>,
}

impl<'a> BodyBulk<'a> {
	#[inline]
	pub fn borrowed(changes: &HashMap<&'a Url, &'a Url>) -> Body<'a> {
		let iter = changes.iter().map(|(&from, &to)| (Cow::Borrowed(from), Cow::Borrowed(to)));

		Self { changes: iter.collect() }.into()
	}
}

impl BodyBulk<'static> {
	#[inline]
	pub fn owned(changes: &HashMap<&Url, &Url>) -> Body<'static> {
		let changes = changes
			.iter()
			.map(|(&from, &to)| (Cow::Owned(from.clone()), Cow::Owned(to.clone())))
			.collect();

		Self { changes }.into()
	}
}

impl<'a> From<BodyBulk<'a>> for Body<'a> {
	fn from(value: BodyBulk<'a>) -> Self { Self::Bulk(value) }
}

impl IntoLua for BodyBulk<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		BodyBulkIter { inner: self.changes.into_iter() }.into_lua(lua)
	}
}

// --- Iterator
pub struct BodyBulkIter {
	pub inner: hash_map::IntoIter<Cow<'static, Url>, Cow<'static, Url>>,
}

impl UserData for BodyBulkIter {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.inner.len()));

		methods.add_meta_function(MetaMethod::Pairs, |lua, me: AnyUserData| {
			let iter = lua.create_function(|lua, mut me: UserDataRefMut<Self>| {
				if let Some((Cow::Owned(from), Cow::Owned(to))) = me.inner.next() {
					(yazi_binding::Url::new(from), yazi_binding::Url::new(to)).into_lua_multi(lua)
				} else {
					().into_lua_multi(lua)
				}
			})?;

			Ok((iter, me))
		});
	}
}
