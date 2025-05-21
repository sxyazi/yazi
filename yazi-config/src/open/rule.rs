use std::fmt;

use serde::{Deserialize, Deserializer, de::{self, Visitor}};

use crate::pattern::Pattern;

#[derive(Debug, Deserialize)]
pub struct OpenRule {
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	#[serde(deserialize_with = "OpenRule::deserialize")]
	pub r#use: Vec<String>,
}

impl OpenRule {
	#[inline]
	pub fn any_file(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_file()) }

	#[inline]
	pub fn any_dir(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_dir()) }
}

impl OpenRule {
	fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct UseVisitor;

		impl<'de> Visitor<'de> for UseVisitor {
			type Value = Vec<String>;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a string, or array of strings")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: de::SeqAccess<'de>,
			{
				let mut uses = Vec::with_capacity(seq.size_hint().unwrap_or(0));
				while let Some(use_) = seq.next_element::<String>()? {
					uses.push(use_);
				}
				Ok(uses)
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				Ok(vec![value.to_owned()])
			}

			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				Ok(vec![v])
			}
		}

		deserializer.deserialize_any(UseVisitor)
	}
}
