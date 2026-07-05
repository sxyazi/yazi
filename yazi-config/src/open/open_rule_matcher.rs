use std::{borrow::Cow, sync::Arc};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use yazi_binding::Iter;
use yazi_fs::file::{File, FileRef};
use yazi_shared::id::Id;

use crate::{Selectable, YAZI, open::{OpenRule, OpenRuleArc, OpenRules}};

#[derive(Default)]
pub struct OpenRuleMatcher<'a> {
	pub rules:  Arc<Vec<OpenRuleArc>>,
	pub id:     Id,
	pub file:   Option<Cow<'a, File>>,
	pub mime:   Option<Cow<'a, str>>,
	pub all:    bool,
	pub offset: usize,
}

impl From<&OpenRules> for OpenRuleMatcher<'_> {
	fn from(rules: &OpenRules) -> Self {
		Self { rules: rules.load_full(), all: true, ..Default::default() }
	}
}

impl OpenRuleMatcher<'_> {
	pub fn matches(&self, rule: &OpenRule) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			rule.id == self.id
		} else {
			rule.match_with(self.file.as_deref(), self.mime.as_deref())
		}
	}
}

impl Iterator for OpenRuleMatcher<'_> {
	type Item = OpenRuleArc;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(rule) = self.rules.get(self.offset) {
			self.offset += 1;
			if self.matches(rule) {
				return Some(rule.clone());
			}
		}
		None
	}
}

impl TryFrom<Table> for OpenRuleMatcher<'static> {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();
		let file: Option<FileRef> = value.raw_get("file")?;
		let mime: Option<String> = value.raw_get("mime")?;

		Ok(Self {
			rules: YAZI.open.load_full(),
			id,
			file: file.map(TryInto::try_into).transpose()?,
			mime: mime.map(Into::into),
			..Default::default()
		})
	}
}

impl FromLua for OpenRuleMatcher<'static> {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table of OpenRuleMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for OpenRuleMatcher<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { Iter::new(self, None).into_lua(lua) }
}
