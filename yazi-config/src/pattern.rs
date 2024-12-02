use std::{path::Path, str::FromStr};

use globset::GlobBuilder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct Pattern {
	inner:   globset::GlobMatcher,
	is_dir:  bool,
	is_star: bool,
	#[cfg(windows)]
	sep_lit: bool,
}

impl Pattern {
	#[inline]
	pub fn match_mime(&self, mime: impl AsRef<str>) -> bool {
		self.is_star || (!mime.as_ref().is_empty() && self.inner.is_match(mime.as_ref()))
	}

	#[inline]
	pub fn match_path(&self, path: impl AsRef<Path>, is_dir: bool) -> bool {
		if is_dir != self.is_dir {
			return false;
		} else if self.is_star {
			return true;
		}

		#[cfg(windows)]
		let path = if self.sep_lit {
			yazi_fs::backslash_to_slash(path.as_ref())
		} else {
			std::borrow::Cow::Borrowed(path.as_ref())
		};

		self.inner.is_match(path)
	}

	#[inline]
	pub fn any_file(&self) -> bool { self.is_star && !self.is_dir }

	#[inline]
	pub fn any_dir(&self) -> bool { self.is_star && self.is_dir }
}

impl FromStr for Pattern {
	type Err = globset::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let a = s.trim_start_matches("\\s");
		let b = a.trim_end_matches('/');
		let sep_lit = b.contains('/');

		let inner = GlobBuilder::new(b)
			.case_insensitive(a.len() == s.len())
			.literal_separator(sep_lit)
			.backslash_escape(false)
			.empty_alternates(true)
			.build()?
			.compile_matcher();

		Ok(Self {
			inner,
			is_dir: b.len() < a.len(),
			is_star: b == "*",
			#[cfg(windows)]
			sep_lit,
		})
	}
}

impl TryFrom<String> for Pattern {
	type Error = globset::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::from_str(s.as_str()) }
}

#[cfg(test)]
mod tests {
	use super::*;

	fn matches(glob: &str, path: &str) -> bool {
		Pattern::from_str(glob).unwrap().match_path(path, false)
	}

	#[cfg(unix)]
	#[test]
	fn test_unix() {
		// Wildcard
		assert!(matches("*", "/foo"));
		assert!(matches("*", "/foo/bar"));
		assert!(matches("**", "foo"));
		assert!(matches("**", "/foo"));
		assert!(matches("**", "/foo/bar"));

		// Filename
		assert!(matches("*.md", "foo.md"));
		assert!(matches("*.md", "/foo.md"));
		assert!(matches("*.md", "/foo/bar.md"));

		// 1-star
		assert!(matches("/*", "/foo"));
		assert!(matches("/*/*.md", "/foo/bar.md"));

		// 2-star
		assert!(matches("/**", "/foo"));
		assert!(matches("/**", "/foo/bar"));
		assert!(matches("**/**", "/foo"));
		assert!(matches("**/**", "/foo/bar"));
		assert!(matches("/**/*", "/foo"));
		assert!(matches("/**/*", "/foo/bar"));

		// Failures
		assert!(!matches("/*/*", "/foo"));
		assert!(!matches("/*/*.md", "/foo.md"));
		assert!(!matches("/*", "/foo/bar"));
		assert!(!matches("/*.md", "/foo/bar.md"));
	}

	#[cfg(windows)]
	#[test]
	fn test_windows() {
		// Wildcard
		assert!(matches("*", r#"C:\foo"#));
		assert!(matches("*", r#"C:\foo\bar"#));
		assert!(matches("**", r#"foo"#));
		assert!(matches("**", r#"C:\foo"#));
		assert!(matches("**", r#"C:\foo\bar"#));

		// Filename
		assert!(matches("*.md", r#"foo.md"#));
		assert!(matches("*.md", r#"C:\foo.md"#));
		assert!(matches("*.md", r#"C:\foo\bar.md"#));

		// 1-star
		assert!(matches(r#"C:/*"#, r#"C:\foo"#));
		assert!(matches(r#"C:/*/*.md"#, r#"C:\foo\bar.md"#));

		// 2-star
		assert!(matches(r#"C:/**"#, r#"C:\foo"#));
		assert!(matches(r#"C:/**"#, r#"C:\foo\bar"#));
		assert!(matches(r#"**/**"#, r#"C:\foo"#));
		assert!(matches(r#"**/**"#, r#"C:\foo\bar"#));
		assert!(matches(r#"C:/**/*"#, r#"C:\foo"#));
		assert!(matches(r#"C:/**/*"#, r#"C:\foo\bar"#));

		// Drive letter
		assert!(matches(r#"*:/*"#, r#"C:\foo"#));
		assert!(matches(r#"*:/**/*.md"#, r#"C:\foo\bar.md"#));

		// Failures
		assert!(!matches(r#"C:/*/*"#, r#"C:\foo"#));
		assert!(!matches(r#"C:/*/*.md"#, r#"C:\foo.md"#));
		assert!(!matches(r#"C:/*"#, r#"C:\foo\bar"#));
		assert!(!matches(r#"C:/*.md"#, r#"C:\foo\bar.md"#));
	}
}
