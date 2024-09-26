use std::{borrow::Cow, collections::VecDeque, hash::{Hash, Hasher}, sync::OnceLock};

use regex::Regex;
use serde::Deserialize;
use yazi_shared::event::Cmd;

use super::Key;

static RE: OnceLock<Regex> = OnceLock::new();
static PLURAL_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Clone, Copy, Default)]
pub enum Plurality {
	Singular,
	#[default]
	Plural,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chord {
	#[serde(deserialize_with = "super::deserialize_on")]
	pub on:   Vec<Key>,
	#[serde(deserialize_with = "super::deserialize_run")]
	pub run:  Vec<Cmd>,
	pub desc: Option<String>,
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

	pub fn desc(&self, p: Plurality) -> Option<Cow<str>> {
		self.desc.as_ref().map(|s| {
			let pluar_re = PLURAL_RE.get_or_init(|| Regex::new(r"\{([^{}]+)\}").unwrap());
			let result = pluar_re.replace_all(s, |caps: &regex::Captures| {
				let parts: Vec<&str> = caps[1].split('|').collect();
				match p {
					Plurality::Singular => parts.get(1).unwrap_or(&"").to_string(),
					Plurality::Plural => parts[0].to_string(),
				}
			});
			let whitespace_re = RE.get_or_init(|| Regex::new(r"\s+").unwrap());
			match result {
				Cow::Owned(result) => Cow::Owned(whitespace_re.replace_all(&result, " ").into_owned()),
				Cow::Borrowed(result) => whitespace_re.replace_all(result, " "),
			}
		})
	}

	pub fn desc_or_run(&self, p: Plurality) -> Cow<str> { self.desc(p).unwrap_or_else(|| self.run().into()) }

	#[inline]
	pub fn contains(&self, s: &str) -> bool {
		let s = s.to_lowercase();
		self.desc(Plurality::default()).map(|d| d.to_lowercase().contains(&s)) == Some(true)
			|| self.run().to_lowercase().contains(&s)
			|| self.on().to_lowercase().contains(&s)
	}

	#[inline]
	pub fn to_seq(&self) -> VecDeque<Cmd> { self.run.iter().map(|c| c.shallow_clone()).collect() }
}
