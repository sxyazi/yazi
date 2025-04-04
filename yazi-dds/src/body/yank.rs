use std::{borrow::Cow, collections::HashSet};

use mlua::{IntoLua, Lua, MetaMethod, UserData, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyYank<'a> {
	pub cut:  bool,
	pub urls: Cow<'a, HashSet<Url>>,
	#[serde(skip)]
	dummy:    bool,
}

impl<'a> BodyYank<'a> {
	#[inline]
	pub fn borrowed(cut: bool, urls: &'a HashSet<Url>) -> Body<'a> {
		Self { cut, urls: Cow::Borrowed(urls), dummy: false }.into()
	}
}

impl BodyYank<'static> {
	#[inline]
	pub fn dummy() -> Body<'static> {
		Self { cut: false, urls: Default::default(), dummy: true }.into()
	}
}

impl<'a> From<BodyYank<'a>> for Body<'a> {
	fn from(value: BodyYank<'a>) -> Self { Self::Yank(value) }
}

impl IntoLua for BodyYank<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		if let Some(Cow::Owned(urls)) = Some(self.urls).filter(|_| !self.dummy) {
			BodyYankIter { cut: self.cut, urls: urls.into_iter().collect() }.into_lua(lua)
		} else {
			lua.create_table()?.into_lua(lua)
		}
	}
}

// --- Iterator
#[derive(Clone)]
pub struct BodyYankIter {
	pub cut:  bool,
	pub urls: Vec<Url>,
}

impl UserData for BodyYankIter {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("cut", |_, me| Ok(me.cut));
	}

	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.urls.len()));

		methods.add_meta_method(MetaMethod::Index, |_, me, idx: usize| {
			Ok(if idx > me.urls.len() || idx == 0 {
				None
			} else {
				Some(yazi_binding::Url::new(me.urls[idx - 1].clone()))
			})
		});
	}
}
