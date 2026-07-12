use std::{ops::{Deref, DerefMut}, sync::Arc};

use hashbrown::HashMap;
use serde::{Deserialize, Deserializer};
use yazi_shared::KebabCasedKey;
use yazi_shim::toml::{DeserializeOverHook, DeserializeOverWith};

use super::Service;

pub struct Services(HashMap<KebabCasedKey, Service>);

impl Deref for Services {
	type Target = HashMap<KebabCasedKey, Service>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Services {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<'de> Deserialize<'de> for Services {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		Self(HashMap::deserialize(deserializer)?)
			.deserialize_over_hook()
			.map_err(serde::de::Error::custom)
	}
}

impl DeserializeOverWith for Services {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(self, de: D) -> Result<Self, D::Error> {
		Ok(Self(self.0.deserialize_over_with(de)?))
	}
}

impl DeserializeOverHook for Services {
	fn deserialize_over_hook(mut self) -> Result<Self, toml::de::Error> {
		for (domain, service) in &mut self.0 {
			let kind = service.kind();
			let auth = Arc::get_mut(service.auth_mut()).expect("unique auth arc");

			auth.kind = kind;
			auth.domain = domain.to_string().into();
		}
		Ok(self)
	}
}
