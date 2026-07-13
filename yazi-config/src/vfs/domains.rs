use std::{ops::{Deref, DerefMut}, sync::Arc};

use hashbrown::HashMap;
use serde::{Deserialize, Deserializer, de::{DeserializeSeed, Error}};
use yazi_shared::{KebabCasedKey, auth::Scheme};

use super::{Service, ServiceSftp};

#[derive(Default)]
pub struct Domains(pub(super) HashMap<KebabCasedKey, Service>);

impl Deref for Domains {
	type Target = HashMap<KebabCasedKey, Service>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Domains {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Domains {
	fn init(&mut self, scheme: &Scheme) {
		for (domain, service) in &mut self.0 {
			let kind = service.kind();
			let auth = Arc::get_mut(service.auth_mut()).expect("unique auth arc");

			auth.kind = kind;
			auth.scheme = scheme.clone();
			auth.domain = domain.clone().into();
		}
	}
}

// --- DomainSeed
pub(super) struct DomainSeed<'a>(pub &'a Scheme);

impl<'de> DeserializeSeed<'de> for DomainSeed<'_> {
	type Value = Domains;

	fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
		let mut domains = Domains(match self.0 {
			Scheme::Regular | Scheme::Search => {
				return Err(D::Error::custom("scheme cannot be configured"));
			}
			Scheme::Sftp => {
				let map = HashMap::<KebabCasedKey, ServiceSftp>::deserialize(deserializer)?;
				map.into_iter().map(|(domain, service)| (domain, Service::Sftp(service))).collect()
			}
			Scheme::Custom(_) => {
				let map = HashMap::<KebabCasedKey, Service>::deserialize(deserializer)?;
				if map.values().any(|service| matches!(service, Service::Sftp(_))) {
					return Err(D::Error::custom("SFTP services must use the `sftp` scheme"));
				}
				map
			}
		});

		domains.init(self.0);
		Ok(domains)
	}
}
