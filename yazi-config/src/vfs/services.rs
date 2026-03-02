use std::ops::Deref;

use anyhow::{Result, bail};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use toml::{Spanned, de::DeTable};
use yazi_codegen::DeserializeOver;

use crate::vfs::Service;

#[derive(Deserialize, Serialize, DeserializeOver)]
pub struct Services(HashMap<String, Service>);

impl Deref for Services {
	type Target = HashMap<String, Service>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Services {
	pub(super) fn reshape(mut self) -> Result<Self> {
		for (name, service) in &mut self.0 {
			if name.is_empty() || name.len() > 20 {
				bail!("VFS name `{name}` must be between 1 and 20 characters");
			} else if !name.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-')) {
				bail!("VFS name `{name}` must be in kebab-case");
			}

			service.reshape()?;
		}

		Ok(self)
	}

	pub(super) fn deserialize_over_with<'de>(
		mut self,
		table: Spanned<DeTable<'de>>,
	) -> Result<Self, toml::de::Error> {
		for (key, value) in table.into_inner() {
			self.0.insert(key.into_inner().into_owned(), <_>::deserialize(value.into_deserializer())?);
		}

		Ok(self)
	}
}
