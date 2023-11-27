use std::borrow::Cow;

use serde::Deserialize;
use yazi_shared::event::Exec;

use super::Key;

#[derive(Debug, Deserialize)]
pub struct Control {
	pub on:   Vec<Key>,
	#[serde(deserialize_with = "super::exec_deserialize")]
	pub exec: Vec<Exec>,
	pub desc: Option<String>,
}

impl Control {
	pub fn to_call(&self) -> Vec<Exec> {
		self
			.exec
			.iter()
			.map(|e| Exec {
				cmd: e.cmd.clone(),
				args: e.args.clone(),
				named: e.named.clone(),
				..Default::default()
			})
			.collect()
	}
}

impl Control {
	#[inline]
	pub fn on(&self) -> String { self.on.iter().map(ToString::to_string).collect() }

	#[inline]
	pub fn exec(&self) -> String {
		self.exec.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; ")
	}

	#[inline]
	pub fn desc_or_exec(&self) -> Cow<str> {
		if let Some(ref s) = self.desc { Cow::Borrowed(s) } else { self.exec().into() }
	}

	#[inline]
	pub fn contains(&self, s: &str) -> bool {
		let s = s.to_lowercase();
		self.desc.as_ref().map(|d| d.to_lowercase().contains(&s)) == Some(true)
			|| self.exec().to_lowercase().contains(&s)
			|| self.on().to_lowercase().contains(&s)
	}
}
