use std::borrow::Cow;

use mlua::{IntoLua, Lua, MetaMethod, UserData, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::fs::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyTabs<'a> {
	pub owned:  bool,
	pub cursor: usize,
	pub items:  Vec<BodyTabsItem<'a>>,
}

impl<'a> BodyTabs<'a> {
	#[inline]
	pub fn borrowed(cursor: usize, urls: &[&'a Url]) -> Body<'a> {
		Self {
			owned: false,
			cursor,
			items: urls.iter().map(|&u| BodyTabsItem { url: Cow::Borrowed(u) }).collect(),
		}
		.into()
	}
}

impl BodyTabs<'static> {
	#[inline]
	pub fn owned(cursor: usize) -> Body<'static> {
		Self { owned: false, cursor, items: Default::default() }.into()
	}
}

impl<'a> From<BodyTabs<'a>> for Body<'a> {
	fn from(value: BodyTabs<'a>) -> Self { Self::Tabs(value) }
}

impl IntoLua<'_> for BodyTabs<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value<'_>> {
		if self.owned {
			BodyTabsIter::from(self).into_lua(lua)
		} else {
			lua.create_table_from([("cursor", self.cursor)])?.into_lua(lua)
		}
	}
}

// --- Item
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BodyTabsItem<'a> {
	pub url: Cow<'a, Url>,
}

impl UserData for BodyTabsItem<'static> {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("url", |lua, me| lua.create_any_userdata(me.url.clone()));
	}
}

// --- Iterator
pub struct BodyTabsIter {
	pub cursor: usize,
	pub items:  Vec<BodyTabsItem<'static>>,
}

impl From<BodyTabs<'static>> for BodyTabsIter {
	fn from(value: BodyTabs<'static>) -> Self { Self { cursor: value.cursor, items: value.items } }
}

impl UserData for BodyTabsIter {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("cursor", |_, me| Ok(me.cursor));
	}

	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.items.len()));

		methods.add_meta_method(MetaMethod::Index, |_, me, idx: usize| {
			if idx > me.items.len() || idx == 0 { Ok(None) } else { Ok(Some(me.items[idx - 1].clone())) }
		});
	}
}
