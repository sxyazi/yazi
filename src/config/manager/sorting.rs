use serde::{de::Visitor, Deserialize, Deserializer};

#[derive(Debug, Clone, Copy)]
pub enum SortBy {
	Alphabetical,
	Created,
	Modified,
	Size,
}

impl From<&str> for SortBy {
	fn from(value: &str) -> Self {
		match value {
			"created" => Self::Created,
			"modified" => Self::Modified,
			"size" => Self::Size,
			_ => Self::Alphabetical,
		}
	}
}

impl<'de> Deserialize<'de> for SortBy {
	fn deserialize<D>(deserializer: D) -> Result<SortBy, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct SortByVisitor;

		impl<'de> Visitor<'de> for SortByVisitor {
			type Value = SortBy;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a sort_by string, e.g. modified")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(SortBy::from(value))
			}
		}

		deserializer.deserialize_str(SortByVisitor)
	}
}
