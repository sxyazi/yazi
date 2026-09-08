use std::{ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use hashbrown::HashMap;
use serde::{Deserialize, Deserializer, de::{DeserializeSeed, Error}};
use yazi_shared::{auth::Scheme, domain::Domain};
use yazi_shim::arc_swap::IntoPointee;

use super::{Service, ServiceSftp};

#[derive(Debug)]
pub struct Domains {
	pub(super) scheme: Scheme,
	inner:             ArcSwap<HashMap<Domain<'static>, Service>>,
}

impl Deref for Domains {
	type Target = ArcSwap<HashMap<Domain<'static>, Service>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Domains {
	pub(crate) fn get<'a, D>(&self, domain: D) -> Option<Service>
	where
		D: Into<Domain<'a>>,
	{
		let inner = self.load();
		inner.get(&domain.into()).or_else(|| inner.get(&Domain::CATCHALL)).cloned()
	}

	pub(crate) fn insert<'a, D>(&self, domain: D, service: Service)
	where
		D: Into<Domain<'a>>,
	{
		let domain = domain.into();
		self.rcu(|inner| {
			let mut next = HashMap::clone(inner);
			next.insert(domain.to_static(), service.clone());
			next
		});
	}

	pub(crate) fn remove<'a, D>(&self, domain: D)
	where
		D: Into<Domain<'a>>,
	{
		let domain = domain.into();
		self.rcu(|inner| {
			let mut next = HashMap::clone(inner);
			next.remove(domain.as_ref());
			next
		});
	}

	pub(crate) fn from_unchecked(scheme: Scheme, inner: HashMap<Domain<'static>, Service>) -> Self {
		Self { scheme, inner: inner.into_pointee() }
	}

	pub(crate) fn unwrap_unchecked(self) -> HashMap<Domain<'static>, Service> {
		Arc::try_unwrap(self.inner.into_inner()).expect("unique domains map")
	}
}

// --- DomainSeed
pub(super) struct DomainSeed<'a>(pub &'a Scheme);

impl<'de> DeserializeSeed<'de> for DomainSeed<'_> {
	type Value = Domains;

	fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
		let mut map = match self.0 {
			Scheme::Regular => return Err(D::Error::custom("scheme cannot be configured")),
			Scheme::Sftp => {
				let map = HashMap::<Domain<'static>, ServiceSftp>::deserialize(deserializer)?;
				map.into_iter().map(|(domain, service)| (domain, Service::Sftp(service.into()))).collect()
			}
			Scheme::Custom(_) => HashMap::<Domain<'static>, Service>::deserialize(deserializer)?,
		};

		for (domain, service) in &mut map {
			service.configure(self.0, domain).map_err(D::Error::custom)?;
		}

		Ok(Domains::from_unchecked(self.0.clone(), map))
	}
}
