use std::{borrow::Cow, collections::VecDeque};

use serde::Deserialize;
use yazi_shared::event::Cmd;

use super::Key;

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
	#[inline]
	pub fn on(&self) -> String { self.on.iter().map(ToString::to_string).collect() }

	#[inline]
	pub fn run(&self) -> String {
		self.run.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("; ")
	}

	#[inline]
	pub fn desc_or_run(&self) -> Cow<str> {
		if let Some(ref s) = self.desc { Cow::Borrowed(s) } else { self.run().into() }
	}

	#[inline]
	pub fn contains(&self, s: &str) -> bool {
		let s = s.to_lowercase();
		self.desc.as_ref().map(|d| d.to_lowercase().contains(&s)) == Some(true)
			|| self.run().to_lowercase().contains(&s)
			|| self.on().to_lowercase().contains(&s)
	}
}
