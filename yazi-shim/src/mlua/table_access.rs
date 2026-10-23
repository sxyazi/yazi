use mlua::{Table, Value, table::TablePairs};
use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess};

pub struct TableAccess<'a> {
	pairs: TablePairs<'a, Value, Value>,
	value: Option<Value>,
}

impl<'a> From<&'a Table> for TableAccess<'a> {
	fn from(table: &'a Table) -> Self { Self { pairs: table.pairs(), value: None } }
}

impl<'de> MapAccess<'de> for TableAccess<'_> {
	type Error = mlua::Error;

	fn next_key_seed<K: DeserializeSeed<'de>>(
		&mut self,
		seed: K,
	) -> Result<Option<K::Value>, Self::Error> {
		let Some((key, value)) = self.pairs.next().transpose()? else {
			return Ok(None);
		};

		self.value = Some(value);
		match key {
			Value::Integer(i @ 1..) => {
				seed.deserialize((i - 1).to_string().into_deserializer()).map(Some)
			}
			key => seed.deserialize(mlua::serde::Deserializer::new(key)).map(Some),
		}
	}

	fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Self::Error> {
		let value =
			self.value.take().ok_or_else(|| mlua::Error::DeserializeError("value is missing".into()))?;

		seed.deserialize(mlua::serde::Deserializer::new(value))
	}
}
