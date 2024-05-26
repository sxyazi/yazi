use std::{fmt, str::FromStr};

use anyhow::Result;
use serde::{de::{self, Visitor}, Deserializer};
use yazi_shared::event::Cmd;

pub(super) fn run_deserialize<'de, D>(deserializer: D) -> Result<Vec<Cmd>, D::Error>
where
	D: Deserializer<'de>,
{
	struct RunVisitor;

	impl<'de> Visitor<'de> for RunVisitor {
		type Value = Vec<Cmd>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a `run` string or array of strings within keymap.toml")
		}

		fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: de::SeqAccess<'de>,
		{
			let mut cmds = vec![];
			while let Some(value) = &seq.next_element::<String>()? {
				cmds.push(Cmd::from_str(value).map_err(de::Error::custom)?);
			}
			if cmds.is_empty() {
				return Err(de::Error::custom("`run` within keymap.toml cannot be empty"));
			}
			Ok(cmds)
		}

		fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
		where
			E: de::Error,
		{
			Ok(vec![Cmd::from_str(value).map_err(de::Error::custom)?])
		}
	}

	deserializer.deserialize_any(RunVisitor)
}
