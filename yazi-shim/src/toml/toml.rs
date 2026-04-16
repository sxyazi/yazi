use serde::Deserialize;
use toml::{Spanned, de, de::{DeString, DeTable, Deserializer}};

pub fn deserialize_spanned<'de, T>(value: Spanned<de::DeValue<'de>>) -> Result<T, de::Error>
where
	T: Deserialize<'de>,
{
	#[derive(Deserialize)]
	struct Wrapper<T> {
		value: T,
	}

	let span = value.span();
	let table: DeTable<'de> =
		[(Spanned::new(span.start..span.start, DeString::Borrowed("value")), value)]
			.into_iter()
			.collect();

	Ok(Wrapper::<T>::deserialize(Deserializer::from(Spanned::new(span, table)))?.value)
}
