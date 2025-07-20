use std::{borrow::Cow, collections::HashSet};

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_parser::mgr::UpdateYankedOpt;
use yazi_shared::url::Url;

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyYank<'a>(UpdateYankedOpt<'a>);

impl<'a> BodyYank<'a> {
	pub fn borrowed(cut: bool, urls: &'a HashSet<Url>) -> Body<'a> {
		Self(UpdateYankedOpt { cut, urls: Cow::Borrowed(urls) }).into()
	}
}

impl BodyYank<'static> {
	pub fn owned(cut: bool, _: &HashSet<Url>) -> Body<'static> {
		Self(UpdateYankedOpt { cut, urls: Default::default() }).into()
	}
}

impl<'a> From<BodyYank<'a>> for Body<'a> {
	fn from(value: BodyYank<'a>) -> Self { Self::Yank(value) }
}

impl IntoLua for BodyYank<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.0.into_lua(lua) }
}
