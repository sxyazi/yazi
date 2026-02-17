use std::{fmt, str::FromStr};

use anyhow::Result;
use serde::{Deserializer, de::{self, Visitor}};
use yazi_shared::event::Action;

use crate::keymap::Key;

pub(super) fn deserialize_on<'de, D>(deserializer: D) -> Result<Vec<Key>, D::Error>
where
	D: Deserializer<'de>,
{
	struct OnVisitor;

	impl<'de> Visitor<'de> for OnVisitor {
		type Value = Vec<Key>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a `on` string or array of strings within keymap.toml")
		}

		fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: de::SeqAccess<'de>,
		{
			let mut keys = Vec::with_capacity(seq.size_hint().unwrap_or(0));
			while let Some(value) = &seq.next_element::<String>()? {
				keys.push(Key::from_str(value).map_err(de::Error::custom)?);
			}
			if keys.is_empty() {
				return Err(de::Error::custom("`on` within keymap.toml cannot be empty"));
			}
			Ok(keys)
		}

		fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
		where
			E: de::Error,
		{
			Ok(vec![Key::from_str(value).map_err(de::Error::custom)?])
		}
	}

	deserializer.deserialize_any(OnVisitor)
}

pub(super) fn deserialize_run<'de, D>(deserializer: D) -> Result<Vec<Action>, D::Error>
where
	D: Deserializer<'de>,
{
	struct RunVisitor;

	impl<'de> Visitor<'de> for RunVisitor {
		type Value = Vec<Action>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a `run` string or array of strings within keymap.toml")
		}

		fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: de::SeqAccess<'de>,
		{
			let mut actions = Vec::with_capacity(seq.size_hint().unwrap_or(0));
			while let Some(value) = &seq.next_element::<String>()? {
				actions.push(Action::from_str(value).map_err(de::Error::custom)?);
			}
			if actions.is_empty() {
				return Err(de::Error::custom("`run` within keymap.toml cannot be empty"));
			}
			Ok(actions)
		}

		fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
		where
			E: de::Error,
		{
			Ok(vec![Action::from_str(value).map_err(de::Error::custom)?])
		}
	}

	deserializer.deserialize_any(RunVisitor)
}
