use toml::{Spanned, de::{self, DeTable}};

pub trait DeserializeOver: DeserializeOverWith + DeserializeOverHook {
	fn deserialize_over(self, input: &str) -> Result<Self, de::Error> {
		self.deserialize_over_with(DeTable::parse(input)?).map_err(|mut err| {
			err.set_input(Some(input));
			err
		})
	}
}

pub trait DeserializeOverWith: Sized {
	fn deserialize_over_with<'de>(self, table: Spanned<DeTable<'de>>) -> Result<Self, de::Error>;
}

pub trait DeserializeOverHook: Sized {
	fn deserialize_over_hook(self) -> Result<Self, de::Error> { Ok(self) }
}

impl<T> DeserializeOver for T where T: DeserializeOverWith + DeserializeOverHook {}
