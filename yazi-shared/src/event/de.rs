use serde::de::value::MapDeserializer;

use super::Action;
use crate::data::KeyDeserializer;

impl<'de> serde::Deserializer<'de> for &'de Action {
	type Error = serde::de::value::Error;

	serde::forward_to_deserialize_any! {
		bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
		unit struct unit_struct newtype_struct seq tuple tuple_struct enum identifier
	}

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_map(visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_map(MapDeserializer::new(
			self.args.iter().map(|(key, value)| (KeyDeserializer::Borrowed(key), value)),
		))
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_some(self)
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_unit()
	}
}
