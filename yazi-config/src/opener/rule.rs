use anyhow::{Result, bail};
use serde::Deserialize;
use yazi_fs::Splatter;

use crate::Platform;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpenerRule {
	pub run:    String,
	#[serde(default)]
	pub block:  bool,
	#[serde(default)]
	pub orphan: bool,
	#[serde(default)]
	pub desc:   String,
	#[serde(default)]
	pub r#for:  Platform,
	#[serde(skip)]
	pub spread: bool,
}

impl OpenerRule {
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
			self.spread =
				Splatter::<()>::spread(&self.run) || self.run.contains("$@") || self.run.contains("$*");
		}
		#[cfg(windows)]
		{
			self.spread = Splatter::<()>::spread(&self.run) || self.run.contains("%*");
		}

		Ok(self)
	}
}
