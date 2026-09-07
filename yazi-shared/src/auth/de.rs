use serde::{Deserializer, de::{self, IntoDeserializer, MapAccess}};

use super::{AuthArc, View};
use crate::{domain::Domain, pool::{InternStr, Symbol}, wire::Wire};

// --- AuthDeserializer
pub(crate) struct AuthDeserializer(pub(crate) AuthArc);

impl<'de> Deserializer<'de> for AuthDeserializer {
	type Error = de::value::Error;

	serde::forward_to_deserialize_any! {
		bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
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
		visitor.visit_map(MapDeserializer {
			kind: Some(self.0.kind.into()),
			scheme: Some(self.0.scheme.intern()),
			domain: Some(self.0.domain.clone()),
			parent: self.0.parent.clone(),
			..Default::default()
		})
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_some(self)
	}
}

// --- ViewDeserializer
pub(crate) struct ViewDeserializer(pub(crate) View);

impl<'de> Deserializer<'de> for ViewDeserializer {
	type Error = de::value::Error;

	serde::forward_to_deserialize_any! {
		bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
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
		visitor.visit_map(MapDeserializer {
			source: Some(self.0.source),
			data: Some(self.0.data),
			..Default::default()
		})
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_some(self)
	}
}

// --- Map
#[derive(Default)]
struct MapDeserializer {
	kind:   Option<&'static str>,
	scheme: Option<Symbol<str>>,
	domain: Option<Domain<'static>>,
	parent: Option<AuthArc>,
	source: Option<AuthArc>,
	data:   Option<Wire>,
}

impl<'de> MapAccess<'de> for MapDeserializer {
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
		} else if self.parent.is_some() {
			Some("parent")
		} else if self.source.is_some() {
			Some("source")
		} else if self.data.is_some() {
			Some("data")
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
		if let Some(parent) = self.parent.take() {
			return seed.deserialize(AuthDeserializer(parent));
		}
		if let Some(source) = self.source.take() {
			return seed.deserialize(AuthDeserializer(source));
		}
		if let Some(data) = self.data.take() {
			return seed.deserialize(data.into_deserializer());
		}

		Err(de::Error::custom("value missing for key"))
	}

	fn size_hint(&self) -> Option<usize> {
		Some(
			self.kind.is_some() as usize
				+ self.scheme.is_some() as usize
				+ self.domain.is_some() as usize
				+ self.parent.is_some() as usize
				+ self.source.is_some() as usize
				+ self.data.is_some() as usize,
		)
	}
}
