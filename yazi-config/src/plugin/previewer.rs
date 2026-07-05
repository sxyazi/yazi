use std::{borrow::Cow, ops::Deref, sync::Arc};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use serde::Deserialize;
use yazi_binding::Iter;
use yazi_fs::file::{File, FileRef};
use yazi_shared::{event::Cmd, id::Id};

use crate::{Mixable, Pattern, Selectable, Selector, YAZI, plugin::{PreviewerArc, Previewers, previewer_id}};

#[derive(Debug, Deserialize)]
pub struct Previewer {
	#[serde(skip, default = "previewer_id")]
	pub id:       Id,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Cmd,
}

impl Deref for Previewer {
	type Target = Cmd;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Previewer {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Previewer {
	fn any_file(&self) -> bool { self.selector.any_file() }

	fn any_dir(&self) -> bool { self.selector.any_dir() }
}

// --- Matcher
#[derive(Default)]
pub struct PreviewerMatcher<'a> {
	pub previewers: Arc<Vec<PreviewerArc>>,
	pub id:         Id,
	pub file:       Option<Cow<'a, File>>,
	pub mime:       Option<Cow<'a, str>>,
	pub all:        bool,
	pub offset:     usize,
}

impl From<&Previewers> for PreviewerMatcher<'_> {
	fn from(previewers: &Previewers) -> Self {
		Self { previewers: previewers.load_full(), all: true, ..Default::default() }
	}
}

impl PreviewerMatcher<'_> {
	pub fn matches(&self, previewer: &Previewer) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			previewer.id == self.id
		} else {
			previewer.match_with(self.file.as_deref(), self.mime.as_deref())
		}
	}
}

impl Iterator for PreviewerMatcher<'_> {
	type Item = PreviewerArc;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(previewer) = self.previewers.get(self.offset) {
			self.offset += 1;
			if self.matches(previewer) {
				return Some(previewer.clone());
			}
		}
		None
	}
}

impl TryFrom<Table> for PreviewerMatcher<'static> {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();
		let file: Option<FileRef> = value.raw_get("file")?;
		let mime: Option<String> = value.raw_get("mime")?;

		Ok(Self {
			previewers: YAZI.plugin.previewers.load_full(),
			id,
			file: file.map(TryInto::try_into).transpose()?,
			mime: mime.map(Into::into),
			..Default::default()
		})
	}
}

impl FromLua for PreviewerMatcher<'static> {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table of PreviewerMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for PreviewerMatcher<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { Iter::new(self, None).into_lua(lua) }
}
