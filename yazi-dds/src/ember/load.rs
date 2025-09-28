use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_fs::FolderStage;
use yazi_shared::{Id, url::UrlBuf};

use super::Ember;

#[derive(Debug, Deserialize, Serialize)]
pub struct EmberLoad<'a> {
	pub tab:   Id,
	pub url:   Cow<'a, UrlBuf>,
	pub stage: Cow<'a, FolderStage>,
}

impl<'a> EmberLoad<'a> {
	pub fn borrowed(tab: Id, url: &'a UrlBuf, stage: &'a FolderStage) -> Ember<'a> {
		Self { tab, url: url.into(), stage: Cow::Borrowed(stage) }.into()
	}
}

impl EmberLoad<'static> {
	pub fn owned(tab: Id, url: &UrlBuf, stage: &FolderStage) -> Ember<'static> {
		Self { tab, url: url.clone().into(), stage: Cow::Owned(stage.clone()) }.into()
	}
}

impl<'a> From<EmberLoad<'a>> for Ember<'a> {
	fn from(value: EmberLoad<'a>) -> Self { Self::Load(value) }
}

impl IntoLua for EmberLoad<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.get().into_lua(lua)?),
				("url", yazi_binding::Url::new(self.url).into_lua(lua)?),
				("stage", yazi_binding::FolderStage::new(self.stage.into_owned()).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
