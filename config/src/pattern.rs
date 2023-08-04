use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct Pattern {
	inner:     glob::Pattern,
	is_folder: bool,
	full_path: bool,
}

impl Pattern {
	#[inline]
	pub fn matches(&self, str: impl AsRef<str>) -> bool { self.inner.matches(str.as_ref()) }

	#[inline]
	pub fn match_path(&self, path: impl AsRef<Path>, is_folder: Option<bool>) -> bool {
		let path = path.as_ref();
		let s = if self.full_path {
			path.to_str()
		} else {
			path.file_name().and_then(|n| n.to_str()).or(path.to_str())
		};
		is_folder.map_or(true, |f| f == self.is_folder) && s.map_or(false, |s| self.matches(s))
	}
}

impl TryFrom<&str> for Pattern {
	type Error = anyhow::Error;

	fn try_from(s: &str) -> Result<Self, Self::Error> {
		let new = s.trim_end_matches('/');
		Ok(Self {
			inner:     glob::Pattern::new(new)?,
			is_folder: new.len() < s.len(),
			full_path: new.contains('/'),
		})
	}
}

impl TryFrom<String> for Pattern {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::try_from(s.as_str()) }
}
