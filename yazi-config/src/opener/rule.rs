use std::sync::Arc;

use serde::Deserialize;
use yazi_fs::Splatter;
use yazi_shared::{Id, NonEmptyString};

use crate::{Platform, opener::OpenerRules, plugin::opener_rule_id};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OpenerRule {
	#[serde(skip, default = "opener_rule_id")]
	pub id:     Id,
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

	pub fn fill(&mut self) {
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

// --- Matcher
#[derive(Default)]
pub struct OpenerRuleMatcher {
	pub rules:  Arc<Vec<Arc<OpenerRule>>>,
	pub id:     Id,
	pub all:    bool,
	pub offset: usize,
}

impl From<&OpenerRules> for OpenerRuleMatcher {
	fn from(rules: &OpenerRules) -> Self {
		Self { rules: rules.load_full(), all: true, ..Default::default() }
	}
}

impl OpenerRuleMatcher {
	pub fn matches(&self, rule: &OpenerRule) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			rule.id == self.id
		} else {
			false
		}
	}
}

impl Iterator for OpenerRuleMatcher {
	type Item = Arc<OpenerRule>;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(rule) = self.rules.get(self.offset) {
			self.offset += 1;
			if self.matches(rule) {
				return Some(rule.clone());
			}
		}
		None
	}
}
