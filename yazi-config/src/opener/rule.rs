use serde::Deserialize;
use yazi_fs::Splatter;
use yazi_shared::NonEmptyString;

use crate::Platform;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpenerRule {
	pub run:    NonEmptyString,
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

	pub(super) fn fill(&mut self) {
		#[cfg(unix)]
		{
			self.spread =
				Splatter::<()>::spread(&self.run) || self.run.contains("$@") || self.run.contains("$*");
		}
		#[cfg(windows)]
		{
			self.spread = Splatter::<()>::spread(&self.run) || self.run.contains("%*");
		}
	}
}
