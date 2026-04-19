use std::ops::Deref;

use anyhow::Result;
use hashbrown::HashMap;
use serde::{Deserialize, Deserializer, Serialize, de, de::{MapAccess, Visitor}};
use yazi_codegen::DeserializeOver;
use yazi_shim::toml::DeserializeOverWith;

use crate::vfs::Service;

#[derive(Serialize, Deserialize, DeserializeOver)]
pub struct Services(HashMap<String, Service>);

impl Deref for Services {
	type Target = HashMap<String, Service>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DeserializeOverWith for Services {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(self, de: D) -> Result<Self, D::Error> {
		struct V(Services);

		impl<'de> Visitor<'de> for V {
			type Value = Services;

			fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				f.write_str("a map of VFS services")
			}

			fn visit_map<M: MapAccess<'de>>(mut self, mut map: M) -> Result<Self::Value, M::Error> {
				while let Some(key) = map.next_key::<String>()? {
					if key.is_empty() || key.len() > 20 {
						return Err(de::Error::custom(format!(
							"VFS name `{key}` must be between 1 and 20 characters"
						)));
					} else if !key.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-')) {
						return Err(de::Error::custom(format!("VFS name `{key}` must be in kebab-case")));
					}
					self.0.0.insert(key, map.next_value()?);
				}
				Ok(self.0)
			}
		}

		de.deserialize_map(V(self))
	}
}
