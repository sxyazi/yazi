use serde::{Deserializer, de::{self, Error, IntoDeserializer, MapAccess, SeqAccess}};

use crate::data::{BytesDeserializer, Data, DataKey, KeyDeserializer};

impl<'de> Deserializer<'de> for &'de Data {
	type Error = de::value::Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Data::Nil => visitor.visit_unit(),
			Data::Boolean(b) => visitor.visit_bool(*b),
			Data::Integer(i) => visitor.visit_i64(*i),
			Data::Number(n) => visitor.visit_f64(*n),
			Data::String(s) => visitor.visit_borrowed_str(s),
			Data::List(l) => visitor.visit_seq(SeqDeserializer { iter: l.iter() }),
			Data::Dict(d) => visitor.visit_map(MapDeserializer { iter: d.iter(), value: None }),
			Data::Id(i) => visitor.visit_u64(i.get()),
			Data::Url(u) => u.into_deserializer().deserialize_any(visitor),
			Data::Path(_) => Err(Error::custom("path not supported")),
			Data::Bytes(b) => BytesDeserializer(b.into()).deserialize_any(visitor),
			Data::Any(_) => Err(Error::custom("any not supported")),
		}
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_bool(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_i8(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_i16(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_i32(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_i64(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_u8(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_u16(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_u32(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_u64(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_f32(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_f64(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		let s: &str = self.try_into().map_err(Error::custom)?;
		let mut chars = s.chars();
		match (chars.next(), chars.next()) {
			(Some(ch), None) => visitor.visit_char(ch),
			_ => Err(Error::custom("not a char")),
		}
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_borrowed_str(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Data::Url(u) => visitor.visit_newtype_struct(u.into_deserializer()),
			_ => self.deserialize_str(visitor),
		}
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Data::Bytes(b) => BytesDeserializer(b.into()).deserialize_bytes(visitor),
			_ => Err(Error::custom("not bytes")),
		}
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_bytes(visitor)
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Data::Nil => visitor.visit_none(),
			_ => visitor.visit_some(self),
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Data::Nil => visitor.visit_unit(),
			_ => Err(Error::custom("expected unit")),
		}
	}

	fn deserialize_unit_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Data::Nil => visitor.visit_unit(),
			_ => Err(Error::custom("expected unit struct")),
		}
	}

	fn deserialize_newtype_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Data::List(l) => visitor.visit_seq(SeqDeserializer { iter: l.iter() }),
			Data::Bytes(b) => BytesDeserializer(b.into()).deserialize_seq(visitor),
			_ => Err(Error::custom("not a sequence")),
		}
	}

	fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_tuple_struct<V>(
		self,
		_name: &'static str,
		_len: usize,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Data::Dict(d) => visitor.visit_map(MapDeserializer { iter: d.iter(), value: None }),
			Data::Url(u) => u.into_deserializer().deserialize_map(visitor),
			_ => Err(Error::custom("not a map")),
		}
	}

	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_map(visitor)
	}

	fn deserialize_enum<V>(
		self,
		_name: &'static str,
		_variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Data::String(s) => visitor.visit_enum((&**s).into_deserializer()),
			_ => Err(Error::custom("not an enum")),
		}
	}

	fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		Err(Error::custom("identifier not supported"))
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_unit()
	}
}

impl<'de> IntoDeserializer<'de, de::value::Error> for &'de Data {
	type Deserializer = Self;

	fn into_deserializer(self) -> Self::Deserializer { self }
}

// --- Seq
struct SeqDeserializer<'a> {
	iter: std::slice::Iter<'a, Data>,
}

impl<'de> SeqAccess<'de> for SeqDeserializer<'de> {
	type Error = de::value::Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: de::DeserializeSeed<'de>,
	{
		self.iter.next().map(|value| seed.deserialize(value)).transpose()
	}

	fn size_hint(&self) -> Option<usize> { Some(self.iter.len()) }
}

// --- Map
struct MapDeserializer<'a> {
	iter:  hashbrown::hash_map::Iter<'a, DataKey, Data>,
	value: Option<&'a Data>,
}

impl<'de> MapAccess<'de> for MapDeserializer<'de> {
	type Error = de::value::Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: de::DeserializeSeed<'de>,
	{
		let Some((key, value)) = self.iter.next() else { return Ok(None) };
		self.value = Some(value);

		seed.deserialize(KeyDeserializer::Borrowed(key)).map(Some)
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: de::DeserializeSeed<'de>,
	{
		seed.deserialize(self.value.take().ok_or_else(|| Error::custom("value missing for key"))?)
	}

	fn size_hint(&self) -> Option<usize> { Some(self.iter.len()) }
}
