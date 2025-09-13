use serde::{Deserializer as _, de::{EnumAccess, MapAccess, SeqAccess, VariantAccess, value::U32Deserializer}};

use crate::Error;

pub(super) struct Deserializer<'a> {
	input: &'a [u8],
}

impl<'a> Deserializer<'a> {
	pub(super) fn once<'de, T>(input: &'de [u8]) -> Result<T, Error>
	where
		T: serde::Deserialize<'de>,
	{
		let mut de = Deserializer { input };
		let t = T::deserialize(&mut de)?;
		if !de.input.is_empty() {
			return Err(Error::serde("trailing bytes"));
		}
		Ok(t)
	}
}

impl<'de> serde::Deserializer<'de> for &mut Deserializer<'de> {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let len = self.input.len();
		visitor.visit_seq(SeqDeserializer { de: self, remaining: len })
	}

	fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("bool not supported"))
	}

	fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("i8 not supported"))
	}

	fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("i16 not supported"))
	}

	fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("i32 not supported"))
	}

	fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("i64 not supported"))
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let b = *self.input.first().ok_or(Error::serde("u8 not enough"))?;

		self.input = &self.input[1..];
		visitor.visit_u8(b)
	}

	fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("u16 not supported"))
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let b: [u8; 4] = self.input.get(..4).ok_or(Error::serde("u32 not enough"))?.try_into().unwrap();

		self.input = &self.input[4..];
		visitor.visit_u32(u32::from_be_bytes(b))
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let b: [u8; 8] = self.input.get(..8).ok_or(Error::serde("u64 not enough"))?.try_into().unwrap();

		self.input = &self.input[8..];
		visitor.visit_u64(u64::from_be_bytes(b))
	}

	fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("f32 not supported"))
	}

	fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("f64 not supported"))
	}

	fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("char not supported"))
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let len: [u8; 4] =
			self.input.get(..4).ok_or(Error::serde("invalid string length"))?.try_into().unwrap();
		let len = u32::from_be_bytes(len) as usize;

		self.input = &self.input[4..];
		let b = self.input.get(..len).ok_or(Error::serde("string not enough"))?;

		self.input = &self.input[len..];
		visitor.visit_str(str::from_utf8(b).map_err(|e| Error::serde(e.to_string()))?)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let len: [u8; 4] =
			self.input.get(..4).ok_or(Error::serde("invalid bytes length"))?.try_into().unwrap();
		let len = u32::from_be_bytes(len) as usize;
		let b = self.input.get(4..4 + len).ok_or(Error::serde("bytes not enough"))?;

		self.input = &self.input[4 + len..];
		visitor.visit_bytes(b)
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_bytes(visitor)
	}

	fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("option not supported"))
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_unit()
	}

	fn deserialize_unit_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_unit()
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
		let len: [u8; 4] =
			self.input.get(..4).ok_or(Error::serde("invalid seq length"))?.try_into().unwrap();

		self.input = &self.input[4..];
		visitor.visit_seq(SeqDeserializer { de: self, remaining: u32::from_be_bytes(len) as _ })
	}

	fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_seq(SeqDeserializer { de: self, remaining: len })
	}

	fn deserialize_tuple_struct<V>(
		self,
		_name: &'static str,
		len: usize,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_tuple(len, visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_map(MapDeserializer { de: self })
	}

	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_tuple(fields.len(), visitor)
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
		visitor.visit_enum(self)
	}

	fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("identifier not supported"))
	}

	fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(Error::serde("ignored any not supported"))
	}

	fn is_human_readable(&self) -> bool { false }
}

struct SeqDeserializer<'a, 'de: 'a> {
	de:        &'a mut Deserializer<'de>,
	remaining: usize,
}

impl<'de> SeqAccess<'de> for SeqDeserializer<'_, 'de> {
	type Error = Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: serde::de::DeserializeSeed<'de>,
	{
		if self.remaining == 0 {
			Ok(None)
		} else {
			self.remaining -= 1;
			seed.deserialize(&mut *self.de).map(Some)
		}
	}

	fn size_hint(&self) -> Option<usize> { Some(self.remaining) }
}

struct MapDeserializer<'a, 'de: 'a> {
	de: &'a mut Deserializer<'de>,
}

impl<'de> MapAccess<'de> for MapDeserializer<'_, 'de> {
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: serde::de::DeserializeSeed<'de>,
	{
		if self.de.input.is_empty() { Ok(None) } else { seed.deserialize(&mut *self.de).map(Some) }
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::DeserializeSeed<'de>,
	{
		seed.deserialize(&mut *self.de)
	}
}

impl<'de> VariantAccess<'de> for &mut Deserializer<'de> {
	type Error = Error;

	fn unit_variant(self) -> Result<(), Self::Error> { Ok(()) }

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: serde::de::DeserializeSeed<'de>,
	{
		seed.deserialize(self)
	}

	fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		use serde::Deserializer;
		self.deserialize_tuple(len, visitor)
	}

	fn struct_variant<V>(
		self,
		fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_tuple(fields.len(), visitor)
	}
}

impl<'de> EnumAccess<'de> for &mut Deserializer<'de> {
	type Error = Error;
	type Variant = Self;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
	where
		V: serde::de::DeserializeSeed<'de>,
	{
		let b: [u8; 4] =
			self.input.get(..4).ok_or(Error::serde("enum not enough"))?.try_into().unwrap();
		self.input = &self.input[4..];

		let de = U32Deserializer::<Self::Error>::new(u32::from_be_bytes(b));
		Ok((seed.deserialize(de)?, self))
	}
}
