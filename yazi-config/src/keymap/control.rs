use std::{borrow::Cow, collections::VecDeque, sync::OnceLock};

use regex::Regex;
use serde::Deserialize;
use yazi_shared::event::Cmd;

use super::Key;

static RE: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Default, Deserialize)]
pub struct Control {
	#[serde(deserialize_with = "super::deserialize_on")]
	pub on:   Vec<Key>,
	#[serde(deserialize_with = "super::deserialize_run")]
	pub run:  Vec<Cmd>,
	pub desc: Option<String>,
}

impl Control {
	#[inline]
	pub fn to_seq(&self) -> VecDeque<Cmd> { self.run.iter().map(|c| c.shallow_clone()).collect() }
}

impl Control {
	pub fn on(&self) -> String { self.on.iter().map(ToString::to_string).collect() }

	pub fn run(&self) -> String {
		RE.get_or_init(|| Regex::new(r"\s+").unwrap())
			.replace_all(&self.run.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("; "), " ")
			.into_owned()
	}

	pub fn desc(&self) -> Option<Cow<str>> {
		self.desc.as_ref().map(|s| RE.get_or_init(|| Regex::new(r"\s+").unwrap()).replace_all(s, " "))
	}

	pub fn desc_or_run(&self) -> Cow<str> { self.desc().unwrap_or_else(|| self.run().into()) }

	#[inline]
	pub fn contains(&self, s: &str) -> bool {
		let s = s.to_lowercase();
		self.desc().map(|d| d.to_lowercase().contains(&s)) == Some(true)
			|| self.run().to_lowercase().contains(&s)
			|| self.on().to_lowercase().contains(&s)
	}
}
