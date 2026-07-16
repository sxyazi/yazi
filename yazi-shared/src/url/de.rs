use std::borrow::Cow;

use serde::{Deserializer, de::{self, IntoDeserializer, MapAccess}};

use crate::{auth::Domain, data::BytesDeserializer, pool::{InternStr, Symbol}, url::UrlCow};

pub struct UrlDeserializer<'a>(pub(super) UrlCow<'a>);

impl<'de, 'a: 'de> Deserializer<'de> for UrlDeserializer<'a> {
	type Error = de::value::Error;

	serde::forward_to_deserialize_any! {
		bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option
		unit unit_struct struct newtype_struct seq tuple tuple_struct enum identifier ignored_any
	}

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_map(visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_map(MapDeserializer::new(self.0))
	}
}

// --- Map
struct MapDeserializer<'a> {
	kind:   Option<&'static str>,
	scheme: Option<Symbol<str>>,
	domain: Option<Domain<'static>>,
	uri:    Option<usize>,
	urn:    Option<usize>,
	path:   Option<Cow<'a, [u8]>>,
}

impl<'a> MapDeserializer<'a> {
	fn new(url: UrlCow<'a>) -> Self {
		let (spec, path) = url.into_pair();
		let (uri, urn) = spec.ports();

		Self {
			kind:   Some(spec.kind.into()),
			scheme: Some(spec.scheme.intern()),
			domain: Some(spec.domain.clone()),
			uri:    Some(uri),
			urn:    Some(urn),
			path:   Some(path.into_encoded_bytes()),
		}
	}
}

impl<'de, 'a: 'de> MapAccess<'de> for MapDeserializer<'a> {
	type Error = de::value::Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: de::DeserializeSeed<'de>,
	{
		let key = if self.kind.is_some() {
			Some("kind")
		} else if self.scheme.is_some() {
			Some("scheme")
		} else if self.domain.is_some() {
			Some("domain")
		} else if self.uri.is_some() {
			Some("uri")
		} else if self.urn.is_some() {
			Some("urn")
		} else if self.path.is_some() {
			Some("path")
		} else {
			None
		};

		key.map(|key| seed.deserialize(key.into_deserializer())).transpose()
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: de::DeserializeSeed<'de>,
	{
		if let Some(kind) = self.kind.take() {
			return seed.deserialize(kind.into_deserializer());
		}
		if let Some(scheme) = self.scheme.take() {
			return seed.deserialize(scheme.into_deserializer());
		}
		if let Some(domain) = self.domain.take() {
			return seed.deserialize(domain.into_deserializer());
		}
		if let Some(uri) = self.uri.take() {
			return seed.deserialize(uri.into_deserializer());
		}
		if let Some(urn) = self.urn.take() {
			return seed.deserialize(urn.into_deserializer());
		}
		if let Some(path) = self.path.take() {
			return seed.deserialize(BytesDeserializer(path));
		}

		Err(de::Error::custom("value missing for key"))
	}

	fn size_hint(&self) -> Option<usize> {
		Some(
			self.kind.is_some() as usize
				+ self.scheme.is_some() as usize
				+ self.domain.is_some() as usize
				+ self.uri.is_some() as usize
				+ self.urn.is_some() as usize
				+ self.path.is_some() as usize,
		)
	}
}
