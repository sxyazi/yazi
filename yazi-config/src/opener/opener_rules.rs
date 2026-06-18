use std::{ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use serde::Deserialize;
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, toml::DeserializeOverHook, vec::{IndexAtError, VecExt}};

use crate::opener::{OpenerRuleArc, OpenerRuleMatcher};

#[derive(Debug, Default, Deserialize)]
pub struct OpenerRules(ArcSwap<Vec<OpenerRuleArc>>);

impl Deref for OpenerRules {
	type Target = ArcSwap<Vec<OpenerRuleArc>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<OpenerRuleArc>> for OpenerRules {
	fn from(inner: Vec<OpenerRuleArc>) -> Self { Self(inner.into_pointee()) }
}

impl OpenerRules {
	pub fn insert(&self, index: isize, rule: OpenerRuleArc) -> Result<(), IndexAtError> {
		self.0.try_rcu(|rules| {
			let (before, after) = rules.split_at(rules.index_at(index)?);
			Ok(
				before
					.iter()
					.cloned()
					.chain([rule.clone()])
					.chain(after.iter().cloned())
					.collect::<Vec<_>>(),
			)
		})?;

		Ok(())
	}

	pub fn remove(&self, matcher: OpenerRuleMatcher) {
		self.0.rcu(|rules| {
			let mut next = Vec::clone(rules);
			next.retain(|rule| !matcher.matches(rule));
			next
		});
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<OpenerRuleArc> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique opener rules arc")
	}
}

impl DeserializeOverHook for OpenerRules {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		let mut inner = self.unwrap_unchecked();

		inner.retain(|r| r.r#for.matches());
		inner.iter_mut().for_each(|r| Arc::get_mut(r).expect("unique opener rule arc").fill());

		Ok(Self(inner.into_pointee()))
	}
}
