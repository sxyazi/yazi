use std::str::FromStr;

use anyhow::Result;
use globset::GlobBuilder;
use serde::Deserialize;
use yazi_shared::{scheme::{SchemeCow, SchemeRef}, url::UrlBuf};

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct Pattern {
	inner:      globset::GlobMatcher,
	scheme:     PatternScheme,
	pub is_dir: bool,
	is_star:    bool,
	#[cfg(windows)]
	sep_lit:    bool,
}

impl Pattern {
	pub fn match_url(&self, url: impl AsRef<UrlBuf>, is_dir: bool) -> bool {
		let url = url.as_ref();

		if is_dir != self.is_dir {
			return false;
		} else if self.is_star {
			return true;
		} else if !self.scheme.matches(&url.scheme) {
			return false;
		}

		#[cfg(unix)]
		let path = &url.loc;

		#[cfg(windows)]
		let path = if self.sep_lit {
			yazi_fs::path::backslash_to_slash(&url.loc)
		} else {
			std::borrow::Cow::Borrowed(url.loc.as_path())
		};

		self.inner.is_match(path)
	}

	pub fn match_mime(&self, mime: impl AsRef<str>) -> bool {
		self.is_star || (!mime.as_ref().is_empty() && self.inner.is_match(mime.as_ref()))
	}

	#[inline]
	pub fn any_file(&self) -> bool { self.is_star && !self.is_dir }

	#[inline]
	pub fn any_dir(&self) -> bool { self.is_star && self.is_dir }
}

impl FromStr for Pattern {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// Trim leading case-sensitive indicator
		let a = s.trim_start_matches(r"\s");

		// Parse the URL scheme if present
		let (scheme, skip) = PatternScheme::parse(a)?;
		let b = &a[skip..];

		// Trim the ending slash which indicates a directory
		let c = b.trim_end_matches('/');

		// Check whether it's a filename pattern or a full path pattern
		let sep_lit = c.contains('/');

		let inner = GlobBuilder::new(c)
			.case_insensitive(a.len() == s.len())
			.literal_separator(sep_lit)
			.backslash_escape(false)
			.empty_alternates(true)
			.build()?
			.compile_matcher();

		Ok(Self {
			inner,
			scheme,
			is_dir: c.len() < b.len(),
			is_star: c == "*",
			#[cfg(windows)]
			sep_lit,
		})
	}
}

impl TryFrom<String> for Pattern {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::from_str(s.as_str()) }
}

// --- Scheme
#[derive(Debug)]
struct PatternScheme(Option<&'static str>);

impl PatternScheme {
	fn parse(s: &str) -> Result<(Self, usize)> {
		let mut me = Self(None);
		let Some((protocol, _)) = s.split_once("://") else {
			return Ok((me, 0));
		};

		if protocol != "*" {
			me.0 = Some(SchemeCow::parse_kind(protocol.as_bytes())?);
		}

		Ok((me, protocol.len() + 3))
	}

	#[inline]
	fn matches<'a>(&self, scheme: impl Into<SchemeRef<'a>>) -> bool {
		self.0.is_none_or(|s| s == scheme.into().kind())
	}
}

// --- Tests
#[cfg(test)]
mod tests {
	use super::*;

	fn matches(glob: &str, url: &str) -> bool {
		Pattern::from_str(glob).unwrap().match_url(UrlBuf::from_str(url).unwrap(), false)
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
