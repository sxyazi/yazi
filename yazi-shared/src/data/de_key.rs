use std::{borrow::Cow, ops::Deref};

use serde::{Deserializer, de::{self, Error, IntoDeserializer}};

use crate::data::DataKey;

pub(crate) enum KeyDeserializer<'a> {
	Borrowed(&'a DataKey),
	Owned(DataKey),
}

impl Deref for KeyDeserializer<'_> {
	type Target = DataKey;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Borrowed(key) => key,
			Self::Owned(key) => key,
		}
	}
}

impl<'de, 'a: 'de> Deserializer<'de> for KeyDeserializer<'a> {
	type Error = de::value::Error;

	serde::forward_to_deserialize_any! {
		bool i8 i16 i32 i128 u8 u16 u32 u128 f32 f64 char bytes byte_buf
		option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum ignored_any
	}

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Self::Borrowed(DataKey::Boolean(b)) => visitor.visit_bool(*b),
			Self::Borrowed(DataKey::Integer(i)) => visitor.visit_i64(*i),
			Self::Borrowed(DataKey::Number(n)) => visitor.visit_f64(n.0),
			Self::Borrowed(DataKey::String(s)) => visitor.visit_borrowed_str(s),
			Self::Borrowed(DataKey::Id(id)) => visitor.visit_u64(id.get()),
			Self::Borrowed(DataKey::Url(u)) => u.into_deserializer().deserialize_any(visitor),
			Self::Owned(DataKey::Boolean(b)) => visitor.visit_bool(b),
			Self::Owned(DataKey::Integer(i)) => visitor.visit_i64(i),
			Self::Owned(DataKey::Number(n)) => visitor.visit_f64(n.0),
			Self::Owned(DataKey::String(Cow::Borrowed(s))) => visitor.visit_borrowed_str(s),
			Self::Owned(DataKey::String(Cow::Owned(s))) => visitor.visit_string(s),
			Self::Owned(DataKey::Id(id)) => visitor.visit_u64(id.get()),
			Self::Owned(DataKey::Url(u)) => u.into_deserializer().deserialize_any(visitor),
			_ => Err(Error::custom("unsupported map key type")),
		}
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_i64((&*self).try_into().map_err(Error::custom)?)
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_u64((&*self).try_into().map_err(Error::custom)?)
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_identifier(visitor)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Self::Borrowed(DataKey::Url(u)) => visitor.visit_newtype_struct(u.into_deserializer()),
			Self::Owned(DataKey::Url(u)) => visitor.visit_newtype_struct(u.into_deserializer()),
			other => other.deserialize_str(visitor),
		}
	}

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Self::Borrowed(DataKey::Boolean(b)) => {
				visitor.visit_borrowed_str(if *b { "true" } else { "false" })
			}
			Self::Borrowed(DataKey::Integer(i)) => visitor.visit_string(i.to_string()),
			Self::Borrowed(DataKey::Number(n)) => visitor.visit_string(n.0.to_string()),
			Self::Borrowed(DataKey::String(s)) => visitor.visit_borrowed_str(s),
			Self::Borrowed(DataKey::Id(id)) => visitor.visit_string(id.get().to_string()),
			Self::Owned(DataKey::Boolean(b)) => {
				visitor.visit_borrowed_str(if b { "true" } else { "false" })
			}
			Self::Owned(DataKey::Integer(i)) => visitor.visit_string(i.to_string()),
			Self::Owned(DataKey::Number(n)) => visitor.visit_string(n.0.to_string()),
			Self::Owned(DataKey::String(Cow::Borrowed(s))) => visitor.visit_borrowed_str(s),
			Self::Owned(DataKey::String(Cow::Owned(s))) => visitor.visit_string(s),
			Self::Owned(DataKey::Id(id)) => visitor.visit_string(id.get().to_string()),
			_ => Err(Error::custom("unsupported map key type")),
		}
	}
}

impl<'de, 'a: 'de> IntoDeserializer<'de, de::value::Error> for KeyDeserializer<'a> {
	type Deserializer = Self;

	fn into_deserializer(self) -> Self::Deserializer { self }
}
