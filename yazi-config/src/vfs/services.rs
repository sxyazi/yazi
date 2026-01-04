use std::ops::Deref;

use anyhow::{Result, bail};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::vfs::Service;

#[derive(Deserialize, Serialize)]
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

	pub(super) fn deserialize_over<'de, D>(mut self, deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let map: HashMap<String, Service> = HashMap::deserialize(deserializer)?;
		self.0.extend(map);

		Ok(self)
	}
}
