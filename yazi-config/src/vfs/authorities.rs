use std::{ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use hashbrown::HashMap;
use serde::{Deserialize, Deserializer, de::{MapAccess, Visitor}};
use yazi_shared::{auth::{AuthArc, Scheme}, domain::Domain};
use yazi_shim::{arc_swap::IntoPointee, toml::DeserializeOverWith};

use super::{DomainSeed, Domains, DomainsArc};
use crate::vfs::Service;

pub struct Authorities(ArcSwap<HashMap<Scheme, DomainsArc>>);

impl Deref for Authorities {
	type Target = ArcSwap<HashMap<Scheme, DomainsArc>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Authorities {
	pub(crate) fn service(&self, scheme: &Scheme, domain: &Domain<'_>) -> Option<Service> {
		self.load().get(scheme)?.get(domain)
	}

	pub(crate) fn auth(&self, scheme: &Scheme, domain: &Domain<'_>) -> Option<AuthArc> {
		let service = self.service(scheme, domain)?;
		Some(if service.auth().domain.is_catchall() {
			AuthArc::new(service.kind(), scheme.clone(), domain.clone())
		} else {
			service.auth().clone()
		})
	}

	pub(super) fn insert(&self, scheme: &Scheme, domains: &DomainsArc) {
		self.0.rcu(|inner| {
			let mut next = HashMap::clone(inner);
			next.insert(scheme.clone(), domains.clone());
			next
		});
	}

	pub(super) fn remove(&self, scheme: &Scheme) {
		self.0.rcu(|inner| {
			let mut next = HashMap::clone(inner);
			next.remove(scheme);
			next
		});
	}

	fn unwrap_unchecked(self) -> HashMap<Scheme, DomainsArc> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique authorities arc")
	}
}

impl<'de> Deserialize<'de> for Authorities {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		struct V;

		impl<'de> Visitor<'de> for V {
			type Value = Authorities;

			fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				f.write_str("a map of VFS schemes")
			}

			fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
				let mut authorities = HashMap::new();
				while let Some(scheme) = map.next_key()? {
					let domains = map.next_value_seed(DomainSeed(&scheme))?;
					authorities.insert(scheme, domains.into());
				}
				Ok(Authorities(authorities.into_pointee()))
			}
		}

		deserializer.deserialize_map(V)
	}
}

impl DeserializeOverWith for Authorities {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(self, de: D) -> Result<Self, D::Error> {
		let mut inner = self.unwrap_unchecked();

		for (scheme, domains) in Self::deserialize(de)?.unwrap_unchecked() {
			if let Some((k, v)) = inner.remove_entry(&scheme) {
				let mut map = v.unwrap_unchecked().unwrap_unchecked();
				map.extend(domains.unwrap_unchecked().unwrap_unchecked());
				inner.insert(k, Domains::from_unchecked(scheme, map).into());
			} else {
				inner.insert(scheme, domains);
			}
		}

		Ok(Self(inner.into_pointee()))
	}
}
