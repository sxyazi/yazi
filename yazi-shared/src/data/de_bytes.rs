use std::borrow::Cow;

use serde::{Deserializer, de::{self, value::SeqDeserializer}};

pub(crate) struct BytesDeserializer<'a>(pub(crate) Cow<'a, [u8]>);

impl<'de, 'a: 'de> Deserializer<'de> for BytesDeserializer<'a> {
	type Error = de::value::Error;

	serde::forward_to_deserialize_any! {
		bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string unit unit_struct
		map struct enum bytes byte_buf identifier ignored_any
	}

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self.0 {
			Cow::Borrowed(b) => visitor.visit_borrowed_bytes(b),
			Cow::Owned(b) => visitor.visit_byte_buf(b),
		}
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_some(self)
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
		match self.0 {
			Cow::Borrowed(b) => visitor.visit_seq(SeqDeserializer::new(b.iter().copied())),
			Cow::Owned(b) => visitor.visit_seq(SeqDeserializer::new(b.into_iter())),
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
}
