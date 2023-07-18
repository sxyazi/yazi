use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct Pattern {
	inner:     glob::Pattern,
	is_folder: bool,
}

impl Pattern {
	pub fn matches(&self, str: impl AsRef<str>) -> bool { self.inner.matches(str.as_ref()) }

	pub fn match_path(&self, path: impl AsRef<Path>, is_folder: Option<bool>) -> bool {
		is_folder.map_or(true, |f| f == self.is_folder) && self.inner.matches_path(path.as_ref())
	}
}

impl TryFrom<&str> for Pattern {
	type Error = anyhow::Error;

	fn try_from(s: &str) -> Result<Self, Self::Error> {
		let is_folder = s.ends_with('/');
		Ok(Self { inner: glob::Pattern::new(s.trim_end_matches('/'))?, is_folder })
	}
}

impl TryFrom<String> for Pattern {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::try_from(s.as_str()) }
}
