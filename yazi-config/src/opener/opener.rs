use std::{mem, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use hashbrown::HashMap;
use serde::{Deserialize, Deserializer};
use yazi_shim::{arc_swap::IntoPointee, toml::{DeserializeOverHook, DeserializeOverWith}};

use super::{OpenerRule, OpenerRules};
use crate::{open::OpenRule, opener::OpenerRuleMatcher};

#[derive(Debug, Deserialize)]
pub struct Opener(ArcSwap<HashMap<String, Arc<OpenerRules>>>);

impl Deref for Opener {
	type Target = ArcSwap<HashMap<String, Arc<OpenerRules>>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Opener {
	pub fn all(&self, open: Arc<OpenRule>) -> impl Iterator<Item = Arc<OpenerRule>> + use<> {
		let inner = self.0.load_full();
		(0..open.r#use.len())
			.filter_map(move |i| inner.get(&open.r#use[i]).cloned())
			.flat_map(|rules| OpenerRuleMatcher::from(&*rules))
	}

	pub fn first(&self, open: &OpenRule) -> Option<Arc<OpenerRule>> {
		let inner = self.0.load();
		open
			.r#use
			.iter()
			.filter_map(|name| inner.get(name))
			.find_map(|rules| rules.load().first().cloned())
	}

	pub fn block(&self, open: &OpenRule) -> Option<Arc<OpenerRule>> {
		let inner = self.0.load();
		open
			.r#use
			.iter()
			.filter_map(|name| inner.get(name))
			.find_map(|rules| rules.load().iter().find(|r| r.block).cloned())
	}

	pub fn insert(&self, name: &str, rules: &Arc<OpenerRules>) {
		self.0.rcu(|inner| {
			let mut next = HashMap::clone(inner);
			next.insert(name.to_owned(), rules.clone());
			next
		});
	}

	pub fn remove(&self, name: &str) {
		self.0.rcu(|inner| {
			let mut next = HashMap::clone(inner);
			next.remove(name);
			next
		});
	}

	pub(crate) fn unwrap_unchecked(self) -> HashMap<String, Arc<OpenerRules>> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique opener arc")
	}
}

impl DeserializeOverHook for Opener {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		let mut inner = self.unwrap_unchecked();
		for mut rules in inner.values_mut() {
			*rules = Arc::try_unwrap(mem::take(&mut rules))
				.expect("unique opener value arc")
				.deserialize_over_hook()?
				.into();
		}

		Ok(Self(inner.into_pointee()))
	}
}

impl DeserializeOverWith for Opener {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(self, de: D) -> Result<Self, D::Error> {
		Ok(Self(self.0.deserialize_over_with(de)?))
	}
}
