use std::borrow::Cow;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug, Default, Deserialize)]
pub struct FeedOpt<'a> {
	#[serde(alias = "0", default)]
	text: Cow<'a, str>,
}

impl<'a> AsRef<str> for FeedOpt<'a> {
	fn as_ref(&self) -> &str { &self.text }
}

impl<C> FromAction<C> for FeedOpt<'_> {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl<'a> From<String> for FeedOpt<'a> {
	fn from(text: String) -> Self { Self { text: Cow::Owned(text) } }
}

impl<'a> From<&'a str> for FeedOpt<'a> {
	fn from(text: &'a str) -> Self { Self { text: Cow::Borrowed(text) } }
}

impl FromLua for FeedOpt<'_> {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for FeedOpt<'_> {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
