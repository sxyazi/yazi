use std::{mem, sync::Arc};

use hashbrown::{HashMap, hash_map};
use yazi_shared::domain::Domain;

use super::{DomainsArc, Service};

pub struct DomainsMatcher {
	iter:     hash_map::Iter<'static, Domain<'static>, Service>,
	_domains: Arc<HashMap<Domain<'static>, Service>>,
}

impl From<&DomainsArc> for DomainsMatcher {
	fn from(domains: &DomainsArc) -> Self {
		let domains = domains.load_full();

		let iter = unsafe {
			mem::transmute::<
				hash_map::Iter<'_, Domain<'static>, Service>,
				hash_map::Iter<'static, Domain<'static>, Service>,
			>(domains.iter())
		};

		Self { iter, _domains: domains }
	}
}

impl Iterator for DomainsMatcher {
	type Item = (Domain<'static>, Service);

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|(domain, service)| (domain.clone(), service.clone()))
	}
}
