use std::borrow::Cow;

use mlua::{MetaMethod, UserData};
use serde::{Deserialize, Serialize};
use yazi_shared::fs::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyTabs<'a> {
	pub cursor: usize,
	pub items:  Vec<BodyTabsItem<'a>>,
}

impl<'a> BodyTabs<'a> {
	#[inline]
	pub fn borrowed(cursor: usize, urls: &[&'a Url]) -> Body<'a> {
		Self { cursor, items: urls.iter().map(|&u| BodyTabsItem::from(u)).collect() }.into()
	}
}

impl BodyTabs<'static> {
	#[inline]
	pub fn dummy(cursor: usize) -> Body<'static> { Self { cursor, items: Default::default() }.into() }
}

impl<'a> From<BodyTabs<'a>> for Body<'a> {
	fn from(value: BodyTabs<'a>) -> Self { Self::Tabs(value) }
}

impl UserData for BodyTabs<'static> {
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

// --- Item
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BodyTabsItem<'a> {
	pub url: Cow<'a, Url>,
}

impl<'a> From<&'a Url> for BodyTabsItem<'a> {
	fn from(value: &'a Url) -> Self { Self { url: Cow::Borrowed(value) } }
}

impl UserData for BodyTabsItem<'static> {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("url", |lua, me| lua.create_any_userdata(me.url.clone()));
	}
}
