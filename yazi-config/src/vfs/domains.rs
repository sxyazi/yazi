use hashbrown::HashMap;
use serde::{Deserialize, Deserializer, de::{self, DeserializeSeed, Error}};
use yazi_shared::{auth::{AuthArc, Scheme}, domain::Domain};

use super::{Service, ServiceSftp};

#[derive(Default)]
pub struct Domains {
	exact:    HashMap<Domain<'static>, Service>,
	catchall: Option<Service>,
}

impl Domains {
	pub(crate) fn get(&self, domain: &Domain<'_>) -> Option<&Service> {
		self.exact.get(domain.as_ref()).or(self.catchall.as_ref())
	}

	pub(crate) fn extend(&mut self, other: Self) {
		self.exact.extend(other.exact);
		if other.catchall.is_some() {
			self.catchall = other.catchall;
		}
	}

	fn init(&mut self, scheme: &Scheme) {
		for (domain, service) in &mut self.exact {
			*service.auth_mut() = AuthArc::new(service.kind(), scheme.clone(), domain);
		}

		if let Some(service) = &mut self.catchall {
			*service.auth_mut() = AuthArc::new(service.kind(), scheme.clone(), Domain::CATCHALL);
		}
	}

	fn from_map<E>(map: HashMap<Domain<'static>, Service>) -> Result<Self, E>
	where
		E: de::Error,
	{
		let mut domains = Self::default();
		for (domain, service) in map {
			if domain.is_catchall() {
				domains.catchall = Some(service);
				continue;
			}

			if service.kind().is_hub() {
				return Err(E::custom("Hub services require a `*` catch-all domain"));
			}
			domains.exact.insert(domain, service);
		}
		Ok(domains)
	}
}

// --- DomainSeed
pub(super) struct DomainSeed<'a>(pub &'a Scheme);

impl<'de> DeserializeSeed<'de> for DomainSeed<'_> {
	type Value = Domains;

	fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
		let mut domains = match self.0 {
			Scheme::Regular => {
				return Err(D::Error::custom("scheme cannot be configured"));
			}
			Scheme::Sftp => {
				let map = HashMap::<Domain<'static>, ServiceSftp>::deserialize(deserializer)?;
				Domains::from_map(
					map.into_iter().map(|(domain, service)| (domain, Service::Sftp(service))).collect(),
				)?
			}
			Scheme::Custom(_) => {
				let map = HashMap::<Domain<'static>, Service>::deserialize(deserializer)?;
				if map.values().any(|service| matches!(service, Service::Sftp(_))) {
					return Err(D::Error::custom("SFTP services must use the `sftp` scheme"));
				}
				Domains::from_map(map)?
			}
		};

		domains.init(self.0);
		Ok(domains)
	}
}
