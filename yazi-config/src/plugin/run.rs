use std::fmt;

use anyhow::Result;
use serde::{de::{self, Visitor}, Deserializer};
use yazi_shared::event::Cmd;

pub(super) fn run_deserialize<'de, D>(deserializer: D) -> Result<Cmd, D::Error>
where
	D: Deserializer<'de>,
{
	struct RunVisitor;

	impl<'de> Visitor<'de> for RunVisitor {
		type Value = Cmd;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a `run` string or array of strings")
		}

		fn visit_seq<A>(self, _: A) -> Result<Self::Value, A::Error>
		where
			A: de::SeqAccess<'de>,
		{
			Err(de::Error::custom("`run` within [plugin] must be a string"))
		}

		fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
		where
			E: de::Error,
		{
			if value.is_empty() {
				return Err(de::Error::custom("`run` within [plugin] cannot be empty"));
			}
			Ok(Cmd { name: value.to_owned(), ..Default::default() })
		}
	}

	deserializer.deserialize_any(RunVisitor)
}
