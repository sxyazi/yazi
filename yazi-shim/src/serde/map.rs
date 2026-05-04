use std::borrow::Cow;

use serde::{Deserializer, de::{DeserializeSeed, IntoDeserializer, MapAccess, value::MapAccessDeserializer}};

struct SingleMapEntryAccess<'k, 'a, M> {
	key: Option<Cow<'k, str>>,
	map: &'a mut M,
}

impl<'k, 'a, 'de, M: MapAccess<'de>> MapAccess<'de> for SingleMapEntryAccess<'k, 'a, M> {
	type Error = M::Error;

	fn next_key_seed<K: DeserializeSeed<'de>>(
		&mut self,
		seed: K,
	) -> Result<Option<K::Value>, Self::Error> {
		match self.key.take() {
			Some(k) => seed.deserialize(k.into_deserializer()).map(Some),
			None => Ok(None),
		}
	}

	fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Self::Error> {
		self.map.next_value_seed(seed)
	}
}

pub fn single_map_entry<'k, 'a, 'de, K, M>(
	key: K,
	map: &'a mut M,
) -> impl Deserializer<'de, Error = M::Error> + use<'k, 'a, 'de, K, M>
where
	K: Into<Cow<'k, str>>,
	M: MapAccess<'de>,
{
	MapAccessDeserializer::new(SingleMapEntryAccess { key: Some(key.into()), map })
}
