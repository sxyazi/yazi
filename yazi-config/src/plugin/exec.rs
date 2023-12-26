use std::fmt;

use anyhow::Result;
use serde::{de::{self, Visitor}, Deserializer};
use yazi_shared::event::Exec;

pub(super) fn exec_deserialize<'de, D>(deserializer: D) -> Result<Exec, D::Error>
where
	D: Deserializer<'de>,
{
	struct ExecVisitor;

	impl<'de> Visitor<'de> for ExecVisitor {
		type Value = Exec;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a `exec` string or array of strings")
		}

		fn visit_seq<A>(self, _: A) -> Result<Self::Value, A::Error>
		where
			A: de::SeqAccess<'de>,
		{
			Err(de::Error::custom("`exec` within [plugin] must be a string"))
		}

		fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
		where
			E: de::Error,
		{
			if value.is_empty() {
				return Err(de::Error::custom("`exec` within [plugin] cannot be empty"));
			}
			Ok(Exec { cmd: value.to_owned(), ..Default::default() })
		}
	}

	deserializer.deserialize_any(ExecVisitor)
}
