use std::{borrow::Cow, hash::{Hash, Hasher}, sync::OnceLock};

use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use yazi_shared::{Layer, event::Cmd};

use super::Key;

static RE: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Default, Deserialize)]
pub struct Chord {
	#[serde(deserialize_with = "super::deserialize_on")]
	pub on:    Vec<Key>,
	#[serde(deserialize_with = "super::deserialize_run")]
	pub run:   Vec<Cmd>,
	pub desc:  Option<String>,
	pub r#for: Option<String>,
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
		self.desc.as_ref().map(|s| RE.get_or_init(|| Regex::new(r"\s+").unwrap()).replace_all(s, " "))
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

impl Chord {
	pub(super) fn reshape(mut self, layer: Layer) -> Result<Self> {
		for cmd in &mut self.run {
			if cmd.layer == Default::default() {
				cmd.layer = layer;
			}
		}
		Ok(self)
	}
}
