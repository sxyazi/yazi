use std::ops::Deref;

use anyhow::Result;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize, de};
use toml::{Spanned, de::DeTable};
use yazi_codegen::DeserializeOver;
use yazi_shim::toml::{DeserializeOverWith, deserialize_spanned};

use crate::vfs::Service;

#[derive(Serialize, Deserialize, DeserializeOver)]
pub struct Services(HashMap<String, Service>);

impl Deref for Services {
	type Target = HashMap<String, Service>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DeserializeOverWith for Services {
	fn deserialize_over_with<'de>(
		mut self,
		table: Spanned<DeTable<'de>>,
	) -> Result<Self, toml::de::Error> {
		for (key, value) in table.into_inner() {
			let key = key.into_inner();
			if key.is_empty() || key.len() > 20 {
				Err(de::Error::custom(format!("VFS name `{key}` must be between 1 and 20 characters")))?;
			} else if !key.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-')) {
				Err(de::Error::custom(format!("VFS name `{key}` must be in kebab-case")))?;
			} else {
				self.0.insert(key.into_owned(), deserialize_spanned(value)?);
			}
		}

		Ok(self)
	}
}
