use anyhow::{Result, bail};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct OpenerRule {
	pub run:    String,
	#[serde(default)]
	pub block:  bool,
	#[serde(default)]
	pub orphan: bool,
	#[serde(default)]
	pub desc:   String,
	pub r#for:  Option<String>,
	#[serde(skip)]
	pub spread: bool,
}

impl OpenerRule {
	#[inline]
	pub fn desc(&self) -> String {
		if !self.desc.is_empty() {
			self.desc.clone()
		} else if let Some(first) = self.run.split_whitespace().next() {
			first.to_owned()
		} else {
			String::new()
		}
	}
}

impl OpenerRule {
	pub(super) fn reshape(mut self) -> Result<Self> {
		if self.run.is_empty() {
			bail!("[open].rules.*.run cannot be empty.");
		}

		#[cfg(unix)]
		{
			self.spread = self.run.contains("$@") || self.run.contains("$*");
		}
		#[cfg(windows)]
		{
			self.spread = self.run.contains("%*");
		}

		Ok(self)
	}
}
