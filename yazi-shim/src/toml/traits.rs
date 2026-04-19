use serde::{Deserializer, de::DeserializeSeed};
use toml::de;

pub trait DeserializeOver: DeserializeOverWith + DeserializeOverHook {
	fn deserialize_over(self, input: &str) -> Result<Self, de::Error> {
		let table = de::DeTable::parse(input)?;
		let de = de::Deserializer::from(table);
		self.deserialize_over_with(de).map_err(|mut err| {
			err.set_input(Some(input));
			err
		})
	}
}

impl<T> DeserializeOver for T where T: DeserializeOverWith + DeserializeOverHook {}

// --- DeserializeOverWith
pub trait DeserializeOverWith: Sized {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(self, de: D) -> Result<Self, D::Error>;
}

// --- DeserializeOverHook
pub trait DeserializeOverHook: Sized {
	fn deserialize_over_hook(self) -> Result<Self, de::Error> { Ok(self) }
}

// --- DeserializeOverSeed
pub struct DeserializeOverSeed<T>(pub T);

impl<'de, T: DeserializeOverWith> DeserializeSeed<'de> for DeserializeOverSeed<T> {
	type Value = T;

	fn deserialize<D: Deserializer<'de>>(self, de: D) -> Result<T, D::Error> {
		self.0.deserialize_over_with(de)
	}
}
