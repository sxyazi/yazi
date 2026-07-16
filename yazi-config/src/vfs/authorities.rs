use std::sync::Arc;

use hashbrown::HashMap;
use serde::{Deserialize, Deserializer, de::{MapAccess, Visitor}};
use yazi_shared::auth::{Auth, Domain, Scheme};
use yazi_shim::toml::DeserializeOverWith;

use super::{DomainSeed, Domains};
use crate::vfs::Service;

pub struct Authorities(HashMap<Scheme, Domains>);

impl Authorities {
	pub fn service(&self, scheme: &Scheme, domain: &Domain<'_>) -> Option<&Service> {
		self.0.get(scheme)?.get(domain)
	}

	pub fn auth(&self, scheme: &Scheme, domain: &Domain<'_>) -> Option<Arc<Auth>> {
		let service = self.service(scheme, domain)?;
		if service.auth().domain.is_catchall() {
			Some(Auth::new(service.kind(), scheme.clone(), domain.clone()))
		} else {
			Some(service.auth().clone())
		}
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
					authorities.insert(scheme, domains);
				}
				Ok(Authorities(authorities))
			}
		}

		deserializer.deserialize_map(V)
	}
}

impl DeserializeOverWith for Authorities {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(mut self, de: D) -> Result<Self, D::Error> {
		for (scheme, domains) in Self::deserialize(de)?.0 {
			self.0.entry(scheme).or_default().extend(domains);
		}
		Ok(self)
	}
}
