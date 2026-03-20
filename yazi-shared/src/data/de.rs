use serde::de::{Error, IntoDeserializer, MapAccess, SeqAccess};

use crate::data::{Data, DataKey};

impl<'de> serde::Deserializer<'de> for &Data {
	type Error = serde::de::value::Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Data::Nil => visitor.visit_unit(),
			Data::Boolean(b) => visitor.visit_bool(*b),
			Data::Integer(i) => visitor.visit_i64(*i),
			Data::Number(n) => visitor.visit_f64(*n),
			Data::String(s) => visitor.visit_str(s),
			Data::List(l) => visitor.visit_seq(DataSeqAccess { iter: l.iter() }),
			Data::Dict(d) => visitor.visit_map(DataMapAccess { iter: d.iter(), value: None }),
			Data::Id(i) => visitor.visit_u64(i.get()),
			Data::Url(_) => Err(Error::custom("url not supported")),
			Data::Path(_) => Err(Error::custom("path not supported")),
			Data::Bytes(b) => visitor.visit_bytes(b),
			Data::Any(_) => Err(Error::custom("any not supported")),
		}
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_bool(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i8(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i16(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i32(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i64(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u8(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u16(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u32(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u64(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_f32(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_f64(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
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
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_str(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let s: &str = self.try_into().map_err(Error::custom)?;
		visitor.visit_string(s.to_owned())
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_bytes(self.try_into().map_err(Error::custom)?)
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let bytes: &[u8] = self.try_into().map_err(Error::custom)?;
		visitor.visit_byte_buf(bytes.to_vec())
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Data::Nil => visitor.visit_none(),
			_ => visitor.visit_some(self),
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
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
		V: serde::de::Visitor<'de>,
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
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Data::List(l) => visitor.visit_seq(DataSeqAccess { iter: l.iter() }),
			Data::Bytes(b) => visitor.visit_seq(DataByteSeqAccess { iter: b.iter() }),
			_ => Err(Error::custom("not a sequence")),
		}
	}

	fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
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
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		if let Data::Dict(d) = self {
			visitor.visit_map(DataMapAccess { iter: d.iter(), value: None })
		} else {
			Err(Error::custom("not a map"))
		}
	}

	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
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
		V: serde::de::Visitor<'de>,
	{
		match self {
			Data::String(s) => visitor.visit_enum((&**s).into_deserializer()),
			_ => Err(Error::custom("not an enum")),
		}
	}

	fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::custom("identifier not supported"))
	}

	fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::custom("ignored any not supported"))
	}
}

struct DataSeqAccess<'a> {
	iter: std::slice::Iter<'a, Data>,
}

impl<'de> SeqAccess<'de> for DataSeqAccess<'_> {
	type Error = serde::de::value::Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: serde::de::DeserializeSeed<'de>,
	{
		self.iter.next().map(|value| seed.deserialize(value)).transpose()
	}

	fn size_hint(&self) -> Option<usize> { Some(self.iter.len()) }
}

struct DataByteSeqAccess<'a> {
	iter: std::slice::Iter<'a, u8>,
}

impl<'de> SeqAccess<'de> for DataByteSeqAccess<'_> {
	type Error = serde::de::value::Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: serde::de::DeserializeSeed<'de>,
	{
		self.iter.next().map(|value| seed.deserialize((*value).into_deserializer())).transpose()
	}

	fn size_hint(&self) -> Option<usize> { Some(self.iter.len()) }
}

struct DataMapAccess<'a> {
	iter:  hashbrown::hash_map::Iter<'a, DataKey, Data>,
	value: Option<&'a Data>,
}

impl<'de> MapAccess<'de> for DataMapAccess<'_> {
	type Error = serde::de::value::Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: serde::de::DeserializeSeed<'de>,
	{
		let Some((key, value)) = self.iter.next() else { return Ok(None) };
		self.value = Some(value);

		match key {
			DataKey::Boolean(b) => seed.deserialize((*b).into_deserializer()).map(Some),
			DataKey::Integer(i) => seed.deserialize((*i).into_deserializer()).map(Some),
			DataKey::Number(n) => seed.deserialize((*n).into_deserializer()).map(Some),
			DataKey::String(s) => seed.deserialize((&**s).into_deserializer()).map(Some),
			DataKey::Id(id) => seed.deserialize(id.get().into_deserializer()).map(Some),
			_ => Err(Error::custom("unsupported map key type")),
		}
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::DeserializeSeed<'de>,
	{
		seed.deserialize(self.value.take().ok_or_else(|| Error::custom("value missing for key"))?)
	}

	fn size_hint(&self) -> Option<usize> { Some(self.iter.len()) }
}
