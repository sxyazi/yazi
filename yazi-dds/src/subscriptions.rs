use std::ops::Deref;

use foldhash::fast::FixedState;
use hashbrown::{HashMap, HashSet};
use yazi_shim::CompactString;

use crate::Subscription;

pub(crate) struct Subscriptions(HashMap<CompactString, HashSet<Subscription>, FixedState>);

impl Deref for Subscriptions {
	type Target = HashMap<CompactString, HashSet<Subscription>, FixedState>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Subscriptions {
	pub(crate) const fn new() -> Self { Self(HashMap::with_hasher(FixedState::with_seed(0))) }

	pub(crate) fn find(&self, kind: &str, plugin: &str) -> Option<&Subscription> {
		self.0.get(kind)?.get(plugin)
	}

	pub(crate) fn insert(&mut self, subscription: Subscription) -> Option<Subscription> {
		let subscriptions = self.0.entry_ref(&subscription.kind).or_default();
		if subscriptions.contains(&subscription) {
			return None;
		}

		subscriptions.insert(subscription.clone());
		Some(subscription)
	}

	pub(crate) fn remove(&mut self, kind: &str, plugin: &str) -> bool {
		let Some(subscriptions) = self.0.get_mut(kind) else { return false };
		if !subscriptions.remove(plugin) {
			return false;
		} else if subscriptions.is_empty() {
			self.0.remove(kind);
		}
		true
	}

	pub(crate) fn remove_exact(&mut self, subscription: &Subscription) -> bool {
		let Some(current) = self.find(&subscription.kind, &subscription.plugin) else { return false };
		if current.id != subscription.id {
			return false;
		}

		self.remove(&subscription.kind, &subscription.plugin)
	}
}
