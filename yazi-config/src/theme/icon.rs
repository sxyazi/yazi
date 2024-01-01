use std::fmt;

use serde::{de::{self, Visitor}, Deserializer};

use crate::Pattern;

#[derive(Debug)]
pub struct Icon {
	pub name:    Pattern,
	pub display: String,
}

impl Icon {
	pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Icon>, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct IconVisitor;

		impl<'de> Visitor<'de> for IconVisitor {
			type Value = Vec<Icon>;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a icon rule, e.g. \"*.md\"  = \"\"")
			}

			fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
			where
				A: de::MapAccess<'de>,
			{
				let mut icons = vec![];
				while let Some((key, value)) = &map.next_entry::<String, String>()? {
					icons.push(Icon {
						name:    Pattern::try_from(key.clone())
							.map_err(|e| de::Error::custom(e.to_string()))?,
						display: value.clone(),
					});
				}
				Ok(icons)
			}
		}

		deserializer.deserialize_map(IconVisitor)
	}
}
