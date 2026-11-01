use std::{mem, sync::Arc};

use hashbrown::{HashMap, hash_map};
use yazi_shared::auth::{AuthArc, Scheme};

use super::{Authorities, DomainsArc, DomainsMatcher};

pub struct VfsMatcher {
	iter:         hash_map::Iter<'static, Scheme, DomainsArc>,
	_authorities: Arc<HashMap<Scheme, DomainsArc>>,
}

impl From<&Authorities> for VfsMatcher {
	fn from(authorities: &Authorities) -> Self {
		let authorities = authorities.load_full();

		let iter = unsafe {
			mem::transmute::<
				hash_map::Iter<'_, Scheme, DomainsArc>,
				hash_map::Iter<'static, Scheme, DomainsArc>,
			>(authorities.iter())
		};

		Self { iter, _authorities: authorities }
	}
}

impl Iterator for VfsMatcher {
	type Item = (Scheme, DomainsArc);

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|(scheme, domains)| (scheme.clone(), domains.clone()))
	}
}

impl VfsMatcher {
	pub fn domains(self) -> impl Iterator<Item = DomainsMatcher> {
		self.map(|(_, domains)| DomainsMatcher::from(&domains))
	}

	pub fn authorities(self) -> impl Iterator<Item = AuthArc> {
		self.domains().flat_map(DomainsMatcher::authorities)
	}
}
