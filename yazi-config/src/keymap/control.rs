use std::{borrow::Cow, collections::VecDeque};

use serde::{Deserialize, Deserializer};
use yazi_shared::event::Cmd;

use super::Key;

#[derive(Debug, Default)]
pub struct Control {
	pub on:   Vec<Key>,
	pub run:  Vec<Cmd>,
	pub desc: Option<String>,
}

impl Control {
	#[inline]
	pub fn to_seq(&self) -> VecDeque<Cmd> {
		self.run.iter().map(|e| e.clone_without_data()).collect()
	}
}

impl Control {
	#[inline]
	pub fn on(&self) -> String { self.on.iter().map(ToString::to_string).collect() }

	#[inline]
	pub fn run(&self) -> String {
		self.run.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; ")
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

// TODO: remove this once Yazi 0.3 is released
impl<'de> Deserialize<'de> for Control {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		pub struct Shadow {
			pub on:   Vec<Key>,
			pub run:  Option<VecCmd>,
			pub exec: Option<VecCmd>,
			pub desc: Option<String>,
		}

		let shadow = Shadow::deserialize(deserializer)?;

		#[derive(Deserialize)]
		struct VecCmd(#[serde(deserialize_with = "super::run_deserialize")] Vec<Cmd>);

		let Some(run) = shadow.run.or(shadow.exec) else {
			return Err(serde::de::Error::custom("missing field `run` within `[keymap]`"));
		};

		Ok(Self { on: shadow.on, run: run.0, desc: shadow.desc })
	}
}
