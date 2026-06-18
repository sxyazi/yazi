use std::{borrow::Cow, ops::Deref, sync::Arc};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use serde::Deserialize;
use yazi_binding::Iter;
use yazi_fs::file::{File, FileRef};
use yazi_shared::{event::Cmd, id::Id};

use crate::{Mixable, Pattern, Selectable, Selector, YAZI, plugin::{SpotterArc, Spotters, spotter_id}};

#[derive(Debug, Deserialize)]
pub struct Spotter {
	#[serde(skip, default = "spotter_id")]
	pub id:       Id,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Cmd,
}

impl Deref for Spotter {
	type Target = Cmd;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Spotter {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Spotter {
	fn any_file(&self) -> bool { self.selector.any_file() }

	fn any_dir(&self) -> bool { self.selector.any_dir() }
}

// --- Matcher
#[derive(Default)]
pub struct SpotterMatcher<'a> {
	pub spotters: Arc<Vec<SpotterArc>>,
	pub id:       Id,
	pub file:     Option<Cow<'a, File>>,
	pub mime:     Option<Cow<'a, str>>,
	pub all:      bool,
	pub offset:   usize,
}

impl From<&Spotters> for SpotterMatcher<'_> {
	fn from(spotters: &Spotters) -> Self {
		Self { spotters: spotters.load_full(), all: true, ..Default::default() }
	}
}

impl SpotterMatcher<'_> {
	pub fn matches(&self, spotter: &Spotter) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			spotter.id == self.id
		} else {
			spotter.match_with(self.file.as_deref(), self.mime.as_deref())
		}
	}
}

impl Iterator for SpotterMatcher<'_> {
	type Item = SpotterArc;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(spotter) = self.spotters.get(self.offset) {
			self.offset += 1;
			if self.matches(spotter) {
				return Some(spotter.clone());
			}
		}
		None
	}
}

impl TryFrom<Table> for SpotterMatcher<'static> {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();
		let file: Option<FileRef> = value.raw_get("file")?;
		let mime: Option<String> = value.raw_get("mime")?;

		Ok(Self {
			spotters: YAZI.plugin.spotters.load_full(),
			id,
			file: file.map(|f| f.clone().into()),
			mime: mime.map(Into::into),
			..Default::default()
		})
	}
}

impl FromLua for SpotterMatcher<'static> {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table of SpotterMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for SpotterMatcher<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { Iter::new(self, None).into_lua(lua) }
}
