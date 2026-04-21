use std::{hash::{BuildHasher, Hash}, sync::Arc};

use arc_swap::ArcSwap;
use hashbrown::HashMap;
use serde::{Deserialize, Deserializer, de::{DeserializeSeed, MapAccess, Visitor}};
use toml::de;

use crate::arc_swap::IntoPointee;

pub trait DeserializeOver: DeserializeOverWith + DeserializeOverHook {
	fn deserialize_over(self, input: &str) -> Result<Self, de::Error> {
		let table = de::DeTable::parse(input)?;
		let de = de::Deserializer::from(table);
		self.deserialize_over_with(de).map_err(|mut err| {
			err.set_input(Some(input));
			err
		})
	}
}

impl<T> DeserializeOver for T where T: DeserializeOverWith + DeserializeOverHook {}

// --- DeserializeOverWith
pub trait DeserializeOverWith: Sized {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(self, de: D) -> Result<Self, D::Error>;
}

impl<K, V, S> DeserializeOverWith for HashMap<K, V, S>
where
	K: Eq + Hash,
	S: BuildHasher,
	for<'de> K: Deserialize<'de>,
	for<'de> V: Deserialize<'de>,
{
	fn deserialize_over_with<'de, D: Deserializer<'de>>(self, de: D) -> Result<Self, D::Error> {
		struct HashMapVisitor<K, V, S>(HashMap<K, V, S>);

		impl<'de, K, V, S> Visitor<'de> for HashMapVisitor<K, V, S>
		where
			K: Eq + Hash + Deserialize<'de>,
			V: Deserialize<'de>,
			S: BuildHasher,
		{
			type Value = HashMap<K, V, S>;

			fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { f.write_str("a map") }

			fn visit_map<M: MapAccess<'de>>(mut self, mut map: M) -> Result<Self::Value, M::Error> {
				while let Some(key) = map.next_key()? {
					self.0.insert(key, map.next_value()?);
				}
				Ok(self.0)
			}
		}

		de.deserialize_map(HashMapVisitor(self))
	}
}

impl<T: DeserializeOverWith> DeserializeOverWith for ArcSwap<T> {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(self, de: D) -> Result<Self, D::Error> {
		Arc::try_unwrap(self.into_inner())
			.unwrap_or_else(|_| panic!("ArcSwap must have single owner during deserialization"))
			.deserialize_over_with(de)
			.map(IntoPointee::into_pointee)
	}
}

// --- DeserializeOverHook
pub trait DeserializeOverHook: Sized {
	fn deserialize_over_hook(self) -> Result<Self, de::Error> { Ok(self) }
}

impl<T: DeserializeOverHook> DeserializeOverHook for ArcSwap<T> {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		Arc::try_unwrap(self.into_inner())
			.unwrap_or_else(|_| panic!("ArcSwap must have single owner during hook"))
			.deserialize_over_hook()
			.map(IntoPointee::into_pointee)
	}
}

// --- DeserializeOverSeed
pub struct DeserializeOverSeed<T>(pub T);

impl<'de, T: DeserializeOverWith> DeserializeSeed<'de> for DeserializeOverSeed<T> {
	type Value = T;

	fn deserialize<D: Deserializer<'de>>(self, de: D) -> Result<T, D::Error> {
		self.0.deserialize_over_with(de)
	}
}
