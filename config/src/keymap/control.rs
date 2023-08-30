use std::borrow::Cow;

use serde::Deserialize;

use super::{Exec, Key};

#[derive(Clone, Debug, Deserialize)]
pub struct Control {
	pub on:   Vec<Key>,
	#[serde(deserialize_with = "Exec::deserialize")]
	pub exec: Vec<Exec>,
	pub desc: Option<String>,
}

impl Control {
	#[inline]
	pub fn exec(&self) -> String {
		self.exec.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; ")
	}

	#[inline]
	pub fn desc_or_exec(&self) -> Cow<str> {
		if let Some(ref s) = self.desc { Cow::Borrowed(s) } else { self.exec().into() }
	}
}
