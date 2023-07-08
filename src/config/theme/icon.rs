use std::fmt;

use serde::{de::Visitor, Deserializer};

use crate::config::Pattern;

#[derive(Debug)]
pub struct Icon {
	pub name:    Pattern,
	pub display: String,
}

impl Icon {
	pub fn new(name: String, display: String) -> Self {
		Self { name: Pattern::from(name.as_ref()), display }
	}

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
				A: serde::de::MapAccess<'de>,
			{
				let mut icons = Vec::new();
				while let Some((key, value)) = &map.next_entry::<String, String>()? {
					icons.push(Icon::new(key.clone(), value.clone()));
				}
				Ok(icons)
			}
		}

		deserializer.deserialize_map(IconVisitor)
	}
}
