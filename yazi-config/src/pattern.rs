use std::path::Path;

use globset::GlobBuilder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct Pattern {
	inner:     globset::GlobMatcher,
	is_star:   bool,
	is_folder: bool,
}

impl Pattern {
	#[inline]
	pub fn match_mime(&self, str: impl AsRef<str>) -> bool { self.inner.is_match(str.as_ref()) }

	#[inline]
	pub fn match_path(&self, path: impl AsRef<Path>, is_folder: bool) -> bool {
		is_folder == self.is_folder && (self.is_star || self.inner.is_match(path))
	}

	#[inline]
	pub fn any_file(&self) -> bool { self.is_star && !self.is_folder }

	#[inline]
	pub fn any_dir(&self) -> bool { self.is_star && self.is_folder }
}

impl TryFrom<&str> for Pattern {
	type Error = anyhow::Error;

	fn try_from(s: &str) -> Result<Self, Self::Error> {
		let a = s.trim_start_matches("\\s");
		let b = a.trim_end_matches('/');

		let inner = GlobBuilder::new(b)
			.case_insensitive(a.len() == s.len())
			.literal_separator(false)
			.backslash_escape(false)
			.empty_alternates(false)
			.build()?
			.compile_matcher();

		Ok(Self { inner, is_star: b == "*", is_folder: b.len() < a.len() })
	}
}

impl TryFrom<String> for Pattern {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::try_from(s.as_str()) }
}
