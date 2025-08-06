use std::{borrow::Cow, collections::HashSet};

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_parser::mgr::UpdateYankedOpt;
use yazi_shared::url::CovUrl;

use super::Ember;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmberYank<'a>(UpdateYankedOpt<'a>);

impl<'a> EmberYank<'a> {
	pub fn borrowed(cut: bool, urls: &'a HashSet<CovUrl>) -> Ember<'a> {
		Self(UpdateYankedOpt { cut, urls: Cow::Borrowed(urls) }).into()
	}
}

impl EmberYank<'static> {
	pub fn owned(cut: bool, _: &HashSet<CovUrl>) -> Ember<'static> {
		Self(UpdateYankedOpt { cut, urls: Default::default() }).into()
	}
}

impl<'a> From<EmberYank<'a>> for Ember<'a> {
	fn from(value: EmberYank<'a>) -> Self { Self::Yank(value) }
}

impl IntoLua for EmberYank<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.0.into_lua(lua) }
}
