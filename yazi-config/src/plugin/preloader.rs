use std::{borrow::Cow, ops::Deref, sync::Arc};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use serde::Deserialize;
use yazi_binding::Iter;
use yazi_fs::file::{File, FileRef};
use yazi_shared::{event::Cmd, id::Id};

use crate::{Mixable, Pattern, Priority, Selectable, Selector, YAZI, plugin::{PreloaderArc, Preloaders, preloader_id}};

#[derive(Debug, Deserialize)]
pub struct Preloader {
	#[serde(skip, default = "preloader_id")]
	pub id:       Id,
	#[serde(skip)]
	pub idx:      u8,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Cmd,
	#[serde(default)]
	pub next:     bool,
	#[serde(default)]
	pub prio:     Priority,
}

impl Deref for Preloader {
	type Target = Cmd;

	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Preloader {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Preloader {}

// --- Matcher
#[derive(Default)]
pub struct PreloaderMatcher<'a> {
	pub preloaders: Arc<Vec<PreloaderArc>>,
	pub id:         Id,
	pub file:       Option<Cow<'a, File>>,
	pub mime:       Option<Cow<'a, str>>,
	pub all:        bool,
	pub offset:     usize,
	pub stop:       bool,
}

impl From<&Preloaders> for PreloaderMatcher<'_> {
	fn from(preloaders: &Preloaders) -> Self {
		Self { preloaders: preloaders.load_full(), all: true, ..Default::default() }
	}
}

impl PreloaderMatcher<'_> {
	pub fn matches(&self, preloader: &Preloader) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			preloader.id == self.id
		} else {
			preloader.match_with(self.file.as_deref(), self.mime.as_deref())
		}
	}
}

impl Iterator for PreloaderMatcher<'_> {
	type Item = PreloaderArc;

	fn next(&mut self) -> Option<Self::Item> {
		if self.stop && !self.all {
			return None;
		}

		while let Some(preloader) = self.preloaders.get(self.offset) {
			self.offset += 1;
			if self.matches(preloader) {
				self.stop = !preloader.next;
				return Some(preloader.clone());
			}
		}
		None
	}
}

impl TryFrom<Table> for PreloaderMatcher<'static> {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();
		let file: Option<FileRef> = value.raw_get("file")?;
		let mime: Option<String> = value.raw_get("mime")?;

		Ok(Self {
			preloaders: YAZI.plugin.preloaders.load_full(),
			id,
			file: file.map(|f| f.clone().into()),
			mime: mime.map(Into::into),
			..Default::default()
		})
	}
}

impl FromLua for PreloaderMatcher<'static> {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table of PreloaderMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for PreloaderMatcher<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { Iter::new(self, None).into_lua(lua) }
}
