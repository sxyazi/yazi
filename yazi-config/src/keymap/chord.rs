use std::{borrow::Cow, fmt::Write, hash::{Hash, Hasher}, sync::Arc};

use mlua::{ExternalError, FromLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use regex::regex;
use serde::{Deserialize, Deserializer, de};
use serde_with::{DeserializeAs, DisplayFromStr, OneOrMany};
use yazi_codegen::DeserializeOver2;
use yazi_shared::{event::{Actions, deserialize_actions}, id::Id};

use super::{Key, ids::chord_id};
use crate::{Mixable, Platform, keymap::{ChordArc, Chords}};

#[derive(Clone, Debug, Default, Deserialize, DeserializeOver2)]
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

impl AsRef<Self> for Chord {
	fn as_ref(&self) -> &Self { self }
}

impl PartialEq for Chord {
	fn eq(&self, other: &Self) -> bool { self.on == other.on }
}

impl Eq for Chord {}

impl Hash for Chord {
	fn hash<H: Hasher>(&self, state: &mut H) { self.on.hash(state) }
}

impl Chord {
	pub fn on(&self) -> String {
		let mut s = String::new();
		for key in &self.on {
			write!(s, "{key}").unwrap();
		}
		s
	}

	fn run(&self) -> String {
		let mut s = String::new();
		for (i, action) in self.run.iter().enumerate() {
			if i > 0 {
				s.push_str("; ");
			}
			write!(s, "{action}").unwrap();
		}

		match regex!(r"\s{2,}|[^\S ]").replace_all(&s, " ") {
			Cow::Borrowed(_) => s,
			Cow::Owned(new) => new,
		}
	}

	pub fn desc_or_run(&self) -> Cow<'_, str> {
		if self.desc.is_empty() {
			self.run().into()
		} else {
			regex!(r"\s{2,}|[^\S ]").replace_all(&self.desc, " ")
		}
	}

	#[inline]
	fn noop(&self) -> bool {
		matches!(self.run.as_slice(), [action] if action.name == "noop" && action.args.is_empty())
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
	pub(super) chords: Arc<Vec<ChordArc>>,
	pub(super) id:     Id,
	pub(super) all:    bool,
	pub(super) offset: usize,
	pub(super) owner:  Option<&'static Chords>,
}

impl From<Id> for ChordMatcher {
	fn from(id: Id) -> Self { Self { id, ..Default::default() } }
}

impl From<&'static Chords> for ChordMatcher {
	fn from(owner: &'static Chords) -> Self {
		Self { chords: owner.load_full(), all: true, owner: Some(owner), ..Default::default() }
	}
}

impl ChordMatcher {
	pub(crate) fn matches(&self, chord: &Chord) -> bool {
		self.all || self.id.nz().is_some_and(|id| chord.id == id)
	}
}

impl Iterator for ChordMatcher {
	type Item = ChordArc;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(chord) = self.chords.get(self.offset) {
			self.offset += 1;
			if self.matches(chord) {
				return Some(match self.owner {
					Some(owner) => chord.clone_for(owner),
					None => chord.clone(),
				});
			}
		}
		None
	}
}

impl TryFrom<Table> for ChordMatcher {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		Ok(Self { id: value.raw_get("id").unwrap_or_default(), ..Default::default() })
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

impl UserData for ChordMatcher {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method_mut(MetaMethod::Call, |_, me, ()| Ok(me.next()));
	}
}
