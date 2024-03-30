use std::{borrow::Cow, collections::{hash_map, HashMap}};

use mlua::{AnyUserData, IntoLua, IntoLuaMulti, Lua, MetaMethod, UserData, UserDataRefMut, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::fs::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyBulk<'a> {
	pub tab:     usize,
	pub changes: Cow<'a, HashMap<Url, Url>>,
}

impl<'a> BodyBulk<'a> {
	#[inline]
	pub fn borrowed(tab: usize, changes: &'a HashMap<Url, Url>) -> Body<'a> {
		Self { tab, changes: Cow::Borrowed(changes) }.into()
	}
}

impl BodyBulk<'static> {
	#[inline]
	pub fn owned(tab: usize, changes: &HashMap<Url, Url>) -> Body<'static> {
		Self { tab, changes: Cow::Owned(changes.clone()) }.into()
	}
}

impl<'a> From<BodyBulk<'a>> for Body<'a> {
	fn from(value: BodyBulk<'a>) -> Self { Self::Bulk(value) }
}

impl IntoLua<'_> for BodyBulk<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_any_userdata(BodyBulkIter {
				tab:   self.tab,
				inner: self.changes.into_owned().into_iter(),
			})?
			.into_lua(lua)
	}
}

// --- Iterator
pub struct BodyBulkIter {
	pub tab:   usize,
	pub inner: hash_map::IntoIter<Url, Url>,
}

impl UserData for BodyBulkIter {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("tab", |_, me| Ok(me.tab));
	}

	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.inner.len()));

		methods.add_meta_function(MetaMethod::Pairs, |lua, me: AnyUserData| {
			let iter = lua.create_function(|lua, mut me: UserDataRefMut<Self>| {
				if let Some((from, to)) = me.inner.next() {
					(lua.create_any_userdata(from)?, lua.create_any_userdata(to)?).into_lua_multi(lua)
				} else {
					().into_lua_multi(lua)
				}
			})?;

			Ok((iter, me))
		});
	}
}
