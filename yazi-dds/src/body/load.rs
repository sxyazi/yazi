use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_fs::FolderStage;
use yazi_shared::{Id, url::Url};

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyLoad<'a> {
	pub tab:   Id,
	pub url:   Cow<'a, Url>,
	pub stage: FolderStage,
}

impl<'a> BodyLoad<'a> {
	#[inline]
	pub fn borrowed(tab: Id, url: &'a Url, stage: FolderStage) -> Body<'a> {
		Self { tab, url: Cow::Borrowed(url), stage }.into()
	}
}

impl BodyLoad<'static> {
	#[inline]
	pub fn dummy(tab: Id, url: &Url, stage: FolderStage) -> Body<'static> {
		Self { tab, url: Cow::Owned(url.clone()), stage }.into()
	}
}

impl<'a> From<BodyLoad<'a>> for Body<'a> {
	fn from(value: BodyLoad<'a>) -> Self { Self::Load(value) }
}

impl IntoLua for BodyLoad<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.get().into_lua(lua)?),
				("url", yazi_binding::Url::new(self.url.into_owned()).into_lua(lua)?),
				("stage", yazi_binding::FolderStage::new(self.stage).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
