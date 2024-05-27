use std::path::Path;

use globset::GlobBuilder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct Pattern {
	inner:   globset::GlobMatcher,
	is_dir:  bool,
	is_star: bool,
}

impl Pattern {
	#[inline]
	pub fn match_mime(&self, str: impl AsRef<str>) -> bool {
		self.is_star || self.inner.is_match(str.as_ref())
	}

	#[inline]
	pub fn match_path(&self, path: impl AsRef<Path>, is_dir: bool) -> bool {
		is_dir == self.is_dir && (self.is_star || self.inner.is_match(path))
	}

	#[inline]
	pub fn any_file(&self) -> bool { self.is_star && !self.is_dir }

	#[inline]
	pub fn any_dir(&self) -> bool { self.is_star && self.is_dir }
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
			.empty_alternates(true)
			.build()?
			.compile_matcher();

		Ok(Self { inner, is_dir: b.len() < a.len(), is_star: b == "*" })
	}
}

impl TryFrom<String> for Pattern {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::try_from(s.as_str()) }
}
