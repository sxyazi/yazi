use std::{borrow::Cow, collections::BTreeMap, iter};

use serde::{Deserialize, Deserializer, de::{self, EnumAccess, Error, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor, value::{MapAccessDeserializer, MapDeserializer, SeqDeserializer}}};

use super::{Wire, WireKey};
use crate::{id::Id, url::UrlBuf};

impl<'de> Deserialize<'de> for Wire {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct V;

		impl<'de> Visitor<'de> for V {
			type Value = Wire;

			fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				formatter.write_str("wire-safe data")
			}

			fn visit_unit<E>(self) -> Result<Self::Value, E>
			where
				E: Error,
			{
				Ok(Wire::Nil)
			}

			fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
			where
				E: Error,
			{
				Ok(value.into())
			}

			fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
			where
				E: Error,
			{
				Ok(value.into())
			}

			fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
			where
				E: Error,
			{
				value.try_into().map(Wire::Integer).map_err(|_| E::custom("integer out of range"))
			}

			fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
			where
				E: Error,
			{
				Ok(value.into())
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: Error,
			{
				Ok(value.to_owned().into())
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
			where
				E: Error,
			{
				Ok(value.into())
			}

			fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
			where
				E: Error,
			{
				Ok(value.to_owned().into())
			}

			fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
			where
				E: Error,
			{
				Ok(value.into())
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: SeqAccess<'de>,
			{
				let mut values = Vec::new();
				while let Some(value) = seq.next_element()? {
					values.push(value);
				}
				Ok(Wire::List(values))
			}

			fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
			where
				A: MapAccess<'de>,
			{
				let mut values = BTreeMap::new();
				while let Some((key, value)) = map.next_entry::<WireKey, Wire>()? {
					values.insert(key, value);
				}
				Ok(values.into())
			}

			fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
			where
				A: EnumAccess<'de>,
			{
				let (variant, data) = data.variant::<Cow<'de, str>>()?;
				match &*variant {
					"Id" => data.newtype_variant().map(|value: u64| Id::from(value).into()),
					_ => Err(A::Error::custom("unsupported wire data variant")),
				}
			}

			fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
			where
				D: Deserializer<'de>,
			{
				UrlBuf::deserialize(deserializer).map(Into::into)
			}
		}

		deserializer.deserialize_any(V)
	}
}

impl<'de> Deserializer<'de> for Wire {
	type Error = de::value::Error;

	serde::forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
		option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier
		ignored_any
	}

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self {
			Self::Nil => visitor.visit_unit(),
			Self::Boolean(b) => visitor.visit_bool(b),
			Self::Integer(i) => visitor.visit_i64(i),
			Self::Number(n) => visitor.visit_f64(n.0),
			Self::String(s) => visitor.visit_string(s),
			Self::List(l) => visitor.visit_seq(SeqDeserializer::new(l.into_iter())),
			Self::Dict(d) => visitor.visit_map(MapDeserializer::new(d.into_iter())),
			Self::Id(i) => visitor
				.visit_enum(MapAccessDeserializer::new(MapDeserializer::new(iter::once(("Id", i.get()))))),
			Self::Url(u) => visitor.visit_newtype_struct(u.into_deserializer()),
			Self::Bytes(b) => visitor.visit_byte_buf(b),
		}
	}
}

impl<'de> IntoDeserializer<'de, de::value::Error> for Wire {
	type Deserializer = Self;

	fn into_deserializer(self) -> Self::Deserializer { self }
}
