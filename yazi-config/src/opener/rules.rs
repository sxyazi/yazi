use std::{mem, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use hashbrown::{HashMap, hash_map};
use serde::Deserialize;
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, toml::DeserializeOverHook, vec::{IndexAtError, VecExt}};

use super::OpenerRule;
use crate::opener::{Opener, OpenerRuleMatcher};

#[derive(Debug, Default, Deserialize)]
pub struct OpenerRules(ArcSwap<Vec<Arc<OpenerRule>>>);

impl Deref for OpenerRules {
	type Target = ArcSwap<Vec<Arc<OpenerRule>>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<Arc<OpenerRule>>> for OpenerRules {
	fn from(inner: Vec<Arc<OpenerRule>>) -> Self { Self(inner.into_pointee()) }
}

impl OpenerRules {
	pub fn insert(&self, index: isize, rule: Arc<OpenerRule>) -> Result<(), IndexAtError> {
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

	pub(crate) fn unwrap_unchecked(self) -> Vec<Arc<OpenerRule>> {
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

// --- Matcher
pub struct OpenerRulesMatcher {
	iter:    hash_map::Iter<'static, String, Arc<OpenerRules>>,
	_opener: Arc<HashMap<String, Arc<OpenerRules>>>,
}

impl From<&Opener> for OpenerRulesMatcher {
	fn from(opener: &Opener) -> Self {
		let opener = opener.load_full();

		let iter = unsafe {
			mem::transmute::<
				hash_map::Iter<'_, String, Arc<OpenerRules>>,
				hash_map::Iter<'static, String, Arc<OpenerRules>>,
			>(opener.iter())
		};

		Self { iter, _opener: opener }
	}
}

impl Iterator for OpenerRulesMatcher {
	type Item = (String, Arc<OpenerRules>);

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|(name, rules)| (name.clone(), rules.clone()))
	}
}
