use std::path::Path;

use glob::MatchOptions;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct Pattern {
	inner:     glob::Pattern,
	sensitive: bool,
	is_folder: bool,
	full_path: bool,
}

impl Pattern {
	#[inline]
	pub fn matches(&self, str: impl AsRef<str>) -> bool {
		self.inner.matches_with(str.as_ref(), MatchOptions {
			case_sensitive:              self.sensitive,
			require_literal_separator:   false,
			require_literal_leading_dot: false,
		})
	}

	#[inline]
	pub fn match_path(&self, path: impl AsRef<Path>, is_folder: Option<bool>) -> bool {
		let path = path.as_ref();
		let s = if self.full_path {
			path.to_str()
		} else {
			path.file_name().and_then(|n| n.to_str()).or_else(|| path.to_str())
		};
		is_folder.map_or(true, |f| f == self.is_folder) && s.is_some_and(|s| self.matches(s))
	}
}

impl TryFrom<&str> for Pattern {
	type Error = anyhow::Error;

	fn try_from(s: &str) -> Result<Self, Self::Error> {
		let a = s.trim_start_matches("\\s");
		let b = a.trim_end_matches('/');
		Ok(Self {
			inner:     glob::Pattern::new(b)?,
			sensitive: a.len() < s.len(),
			is_folder: b.len() < a.len(),
			full_path: b.contains('/'),
		})
	}
}

impl TryFrom<String> for Pattern {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::try_from(s.as_str()) }
}
