use std::{borrow::Cow, hash::{Hash, Hasher}, sync::{Arc, OnceLock}};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use regex::Regex;
use serde::{Deserialize, Deserializer, de};
use serde_with::{DeserializeAs, DisplayFromStr, OneOrMany};
use yazi_binding::Iter;
use yazi_codegen::DeserializeOver2;
use yazi_shared::{event::{Actions, deserialize_actions}, id::Id};

use super::{Key, ids::chord_id};
use crate::{Mixable, Platform, keymap::{ChordArc, Chords}};

static RE: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Default, Deserialize, DeserializeOver2)]
pub struct Chord {
	#[serde(skip, default = "chord_id")]
	pub id:    Id,
	#[serde(deserialize_with = "deserialize_on")]
	pub on:    Vec<Key>,
	#[serde(deserialize_with = "deserialize_actions")]
	pub run:   Actions,
	#[serde(default)]
	pub desc:  String,
	#[serde(default)]
	pub r#for: Platform,
}

impl Clone for Chord {
	fn clone(&self) -> Self {
		Self {
			id:    chord_id(),
			on:    self.on.clone(),
			run:   self.run.clone(),
			desc:  self.desc.clone(),
			r#for: self.r#for,
		}
	}
}

impl AsRef<Chord> for Chord {
	fn as_ref(&self) -> &Chord { self }
}

impl PartialEq for Chord {
	fn eq(&self, other: &Self) -> bool { self.on == other.on }
}

impl Eq for Chord {}

impl Hash for Chord {
	fn hash<H: Hasher>(&self, state: &mut H) { self.on.hash(state) }
}

impl Chord {
	pub fn on(&self) -> String { self.on.iter().map(ToString::to_string).collect() }

	pub fn run(&self) -> String {
		RE.get_or_init(|| Regex::new(r"\s+").unwrap())
			.replace_all(&self.run.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("; "), " ")
			.into_owned()
	}

	pub fn desc(&self) -> Option<Cow<'_, str>> {
		Some(&self.desc)
			.filter(|s| !s.is_empty())
			.map(|s| RE.get_or_init(|| Regex::new(r"\s+").unwrap()).replace_all(s, " "))
	}

	pub fn desc_or_run(&self) -> Cow<'_, str> { self.desc().unwrap_or_else(|| self.run().into()) }

	pub fn contains(&self, s: &str) -> bool {
		let s = s.to_lowercase();
		self.desc().map(|d| d.to_lowercase().contains(&s)) == Some(true)
			|| self.run().to_lowercase().contains(&s)
			|| self.on().to_lowercase().contains(&s)
	}

	#[inline]
	pub(super) fn noop(&self) -> bool {
		self.run.len() == 1 && self.run[0].name == "noop" && self.run[0].args.is_empty()
	}
}

impl Mixable for Chord {
	fn filter(&self) -> bool { self.r#for.matches() && !self.noop() }
}

fn deserialize_on<'de, D>(deserializer: D) -> Result<Vec<Key>, D::Error>
where
	D: Deserializer<'de>,
{
	let keys: Vec<Key> = OneOrMany::<DisplayFromStr>::deserialize_as(deserializer)?;
	if keys.is_empty() {
		return Err(de::Error::custom("'on' cannot be empty"));
	}
	Ok(keys)
}

// --- Matcher
#[derive(Default)]
pub struct ChordMatcher {
	pub id:  Id,
	pub all: bool,
}

impl ChordMatcher {
	pub fn matches(&self, chord: &Chord) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			chord.id == self.id
		} else {
			false
		}
	}
}

impl TryFrom<Table> for ChordMatcher {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();

		Ok(Self { id, ..Default::default() })
	}
}

impl FromLua for ChordMatcher {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table of ChordMatcher".into_lua_err()),
		}
	}
}

// --- Iter
#[derive(Default)]
pub struct ChordIter {
	pub chords:  Arc<Vec<ChordArc>>,
	pub matcher: ChordMatcher,
	pub offset:  usize,
}

impl From<&Chords> for ChordIter {
	fn from(chords: &Chords) -> Self {
		Self {
			chords: chords.load_full(),
			matcher: ChordMatcher { all: true, ..Default::default() },
			..Default::default()
		}
	}
}

impl Iterator for ChordIter {
	type Item = ChordArc;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(chord) = self.chords.get(self.offset) {
			self.offset += 1;
			if self.matcher.matches(chord) {
				return Some(chord.clone());
			}
		}
		None
	}
}

impl IntoLua for ChordIter {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { Iter::new(self, None).into_lua(lua) }
}
