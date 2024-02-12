use std::{borrow::Cow, collections::VecDeque, ops::Deref};

use serde::Deserialize;
use yazi_shared::event::Cmd;

use super::Key;

#[derive(Debug, Default, Deserialize)]
pub struct Control {
	pub on:   Vec<Key>,
	#[serde(deserialize_with = "super::exec_deserialize")]
	pub exec: Vec<Cmd>,
	pub desc: Option<String>,
}

impl Control {
	#[inline]
	pub fn to_seq(&self) -> VecDeque<Cmd> {
		self.exec.iter().map(|e| e.clone_without_data()).collect()
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

#[derive(Debug)]
pub enum ControlCow {
	Owned(Control),
	Borrowed(&'static Control),
}

impl From<&'static Control> for ControlCow {
	fn from(c: &'static Control) -> Self { Self::Borrowed(c) }
}

impl From<Control> for ControlCow {
	fn from(c: Control) -> Self { Self::Owned(c) }
}

impl Deref for ControlCow {
	type Target = Control;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Owned(c) => c,
			Self::Borrowed(c) => c,
		}
	}
}

impl Default for ControlCow {
	fn default() -> Self { Self::Owned(Control::default()) }
}

impl ControlCow {
	pub fn into_seq(self) -> VecDeque<Cmd> {
		match self {
			Self::Owned(c) => c.exec.into(),
			Self::Borrowed(c) => c.to_seq(),
		}
	}
}
