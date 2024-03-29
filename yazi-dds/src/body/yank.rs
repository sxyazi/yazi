use std::{borrow::Cow, collections::HashSet};

use mlua::{IntoLua, Lua, MetaMethod, UserData, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::fs::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyYank<'a> {
	pub owned: bool,
	pub cut:   bool,
	pub urls:  Cow<'a, HashSet<Url>>,
}

impl<'a> BodyYank<'a> {
	#[inline]
	pub fn borrowed(cut: bool, urls: &'a HashSet<Url>) -> Body<'a> {
		Self { owned: false, cut, urls: Cow::Borrowed(urls) }.into()
	}
}

impl BodyYank<'static> {
	#[inline]
	pub fn owned(cut: bool) -> Body<'static> {
		Self { owned: false, cut, urls: Default::default() }.into()
	}
}

impl<'a> From<BodyYank<'a>> for Body<'a> {
	fn from(value: BodyYank<'a>) -> Self { Self::Yank(value) }
}

impl IntoLua<'_> for BodyYank<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value<'_>> {
		if self.owned {
			BodyYankIter::from(self).into_lua(lua)
		} else {
			lua.create_table_from([("cut", self.cut)])?.into_lua(lua)
		}
	}
}

// --- Iterator
pub struct BodyYankIter {
	pub cut:  bool,
	pub urls: Vec<Url>,
}

impl From<BodyYank<'static>> for BodyYankIter {
	fn from(value: BodyYank) -> Self {
		Self { cut: value.cut, urls: value.urls.into_owned().into_iter().collect() }
	}
}

impl UserData for BodyYankIter {
	fn add_fields<'a, F: mlua::UserDataFields<'a, Self>>(fields: &mut F) {
		fields.add_field_method_get("is_cut", |_, me| Ok(me.cut));
	}

	fn add_methods<'a, M: mlua::UserDataMethods<'a, Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.urls.len()));

		methods.add_meta_method(MetaMethod::Index, |lua, me, idx: usize| {
			if idx > me.urls.len() || idx == 0 {
				Ok(None)
			} else {
				Some(lua.create_any_userdata(me.urls[idx - 1].clone())).transpose()
			}
		});
	}
}
