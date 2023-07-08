use std::path::Path;

use serde::{de::Visitor, Deserialize, Deserializer};

#[derive(Debug)]
pub struct Pattern {
	inner:     glob::Pattern,
	is_folder: bool,
}

impl Pattern {
	pub fn matches(&self, str: &str) -> bool { self.inner.matches(str) }

	pub fn match_path(&self, path: &Path, is_folder: Option<bool>) -> bool {
		is_folder.map_or(true, |f| f == self.is_folder) && self.inner.matches_path(path)
	}
}

impl From<&str> for Pattern {
	fn from(value: &str) -> Self {
		let is_folder = value.ends_with('/');
		Self { inner: glob::Pattern::new(value.trim_end_matches('/')).unwrap_or_default(), is_folder }
	}
}

impl<'de> Deserialize<'de> for Pattern {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct PatternVisitor;

		impl<'de> Visitor<'de> for PatternVisitor {
			type Value = Pattern;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a glob pattern, e.g. *.rs")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Pattern::from(value))
			}
		}

		deserializer.deserialize_str(PatternVisitor)
	}
}
