use std::{borrow::Cow, ops::Deref, sync::Arc};

use hashbrown::HashSet;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use serde::Deserialize;
use yazi_binding::Iter;
use yazi_fs::file::{File, FileRef};
use yazi_shared::{event::Cmd, id::Id};

use crate::{Mixable, Pattern, Priority, Selectable, Selector, YAZI, plugin::{FetcherArc, Fetchers, fetcher_id}};

#[derive(Debug, Deserialize)]
pub struct Fetcher {
	#[serde(skip, default = "fetcher_id")]
	pub id:       Id,
	#[serde(skip)]
	pub idx:      u8,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Cmd,
	#[serde(default)]
	pub prio:     Priority,
	pub group:    String,
}

impl Deref for Fetcher {
	type Target = Cmd;

	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Fetcher {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Fetcher {}

// --- Matcher
#[derive(Default)]
pub struct FetcherMatcher<'a> {
	pub fetchers: Arc<Vec<FetcherArc>>,
	pub id:       Id,
	pub file:     Option<Cow<'a, File>>,
	pub mime:     Option<Cow<'a, str>>,
	pub all:      bool,
	pub offset:   usize,
	pub seen:     HashSet<String>,
}

impl From<&Fetchers> for FetcherMatcher<'_> {
	fn from(fetchers: &Fetchers) -> Self {
		Self { fetchers: fetchers.load_full(), all: true, ..Default::default() }
	}
}

impl FetcherMatcher<'_> {
	pub fn matches(&self, fetcher: &Fetcher) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			fetcher.id == self.id
		} else {
			fetcher.match_with(self.file.as_deref(), self.mime.as_deref())
		}
	}
}

impl Iterator for FetcherMatcher<'_> {
	type Item = FetcherArc;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(fetcher) = self.fetchers.get(self.offset) {
			self.offset += 1;
			if !self.matches(fetcher) {
				continue;
			}
			if self.all || self.seen.insert(fetcher.group.clone()) {
				return Some(fetcher.clone());
			}
		}
		None
	}
}

impl TryFrom<Table> for FetcherMatcher<'static> {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();
		let file: Option<FileRef> = value.raw_get("file")?;
		let mime: Option<String> = value.raw_get("mime")?;

		Ok(Self {
			fetchers: YAZI.plugin.fetchers.load_full(),
			id,
			file: file.map(|f| f.clone().into()),
			mime: mime.map(Into::into),
			..Default::default()
		})
	}
}

impl FromLua for FetcherMatcher<'static> {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table of FetcherMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for FetcherMatcher<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { Iter::new(self, None).into_lua(lua) }
}
