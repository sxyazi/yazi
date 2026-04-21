use std::{borrow::Cow, hash::{Hash, Hasher}, sync::OnceLock};

use regex::Regex;
use serde::{Deserialize, Deserializer, de};
use serde_with::{DeserializeAs, DisplayFromStr, OneOrMany};
use yazi_shared::{Layer, Source, event::Action};

use super::Key;
use crate::{Mixable, Platform};

static RE: OnceLock<Regex> = OnceLock::new();

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Chord<const L: u8 = { Layer::App as u8 }> {
	#[serde(deserialize_with = "deserialize_on")]
	pub on:    Vec<Key>,
	#[serde(deserialize_with = "deserialize_run::<L, _>")]
	pub run:   Vec<Action>,
	#[serde(default)]
	pub desc:  String,
	#[serde(default)]
	pub r#for: Platform,
}

impl<const L: u8> PartialEq for Chord<L> {
	fn eq(&self, other: &Self) -> bool { self.on == other.on }
}

impl<const L: u8> Eq for Chord<L> {}

impl<const L: u8> Hash for Chord<L> {
	fn hash<H: Hasher>(&self, state: &mut H) { self.on.hash(state) }
}

impl<const L: u8> Chord<L> {
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

impl<const L: u8> Mixable for Chord<L> {
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

fn deserialize_run<'de, const L: u8, D>(deserializer: D) -> Result<Vec<Action>, D::Error>
where
	D: Deserializer<'de>,
{
	let mut actions: Vec<Action> = OneOrMany::<DisplayFromStr>::deserialize_as(deserializer)?;

	let Some(layer) = Layer::from_repr(L) else {
		return Err(de::Error::custom(format!("invalid keymap layer const: {L}")));
	};

	for action in &mut actions {
		action.source = Source::Key;
		if action.layer == Layer::Null {
			action.layer = layer;
		}
	}

	Ok(actions)
}
