use std::{fmt::Debug, str::FromStr};

use anyhow::{Result, bail};
use globset::{Candidate, GlobBuilder};
use serde::Deserialize;
use yazi_shared::{scheme::SchemeKind, url::AsUrl};

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct Pattern {
	inner:      globset::GlobMatcher,
	scheme:     PatternScheme,
	pub is_dir: bool,
	is_star:    bool,
	#[cfg(windows)]
	sep_lit:    bool,
}

impl Debug for Pattern {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Pattern")
			.field("regex", &self.inner.glob().regex())
			.field("scheme", &self.scheme)
			.field("is_dir", &self.is_dir)
			.field("is_star", &self.is_star)
			.finish()
	}
}

impl Pattern {
	pub fn match_url(&self, url: impl AsUrl, is_dir: bool) -> bool {
		let url = url.as_url();

		if is_dir != self.is_dir {
			return false;
		} else if !self.scheme.matches(url.kind()) {
			return false;
		} else if self.is_star {
			return true;
		}

		#[cfg(unix)]
		{
			self.inner.is_match_candidate(&Candidate::from_bytes(url.loc().encoded_bytes()))
		}

		#[cfg(windows)]
		if self.sep_lit {
			use yazi_shared::strand::{AsStrand, StrandLike};
			self.inner.is_match_candidate(&Candidate::from_bytes(
				url.loc().as_strand().backslash_to_slash().encoded_bytes(),
			))
		} else {
			self.inner.is_match_candidate(&Candidate::from_bytes(url.loc().encoded_bytes()))
		}
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
#[derive(Clone, Copy, Debug)]
enum PatternScheme {
	Any,
	Local,
	Remote,
	Virtual,

	Regular,
	Search,
	Archive,
	Sftp,
}

impl PatternScheme {
	fn parse(s: &str) -> Result<(Self, usize)> {
		let Some((protocol, _)) = s.split_once("://") else {
			return Ok((Self::Any, 0));
		};

		let scheme = match protocol {
			"*" => Self::Any,
			"local" => Self::Local,
			"remote" => Self::Remote,
			"virtual" => Self::Virtual,

			"regular" => Self::Regular,
			"search" => Self::Search,
			"archive" => Self::Archive,
			"sftp" => Self::Sftp,

			"" => bail!("Invalid URL pattern: protocol is empty"),
			_ => bail!("Unknown protocol in URL pattern: {protocol}"),
		};

		Ok((scheme, protocol.len() + 3))
	}

	#[inline]
	fn matches(self, kind: SchemeKind) -> bool {
		use SchemeKind as K;

		match (self, kind) {
			(Self::Any, _) => true,
			(Self::Local, s) => s.is_local(),
			(Self::Remote, s) => s.is_remote(),
			(Self::Virtual, s) => s.is_virtual(),

			(Self::Regular, K::Regular) => true,
			(Self::Search, K::Search) => true,
			(Self::Archive, K::Archive) => true,
			(Self::Sftp, K::Sftp) => true,

			_ => false,
		}
	}
}

// --- Tests
#[cfg(test)]
mod tests {
	use yazi_shared::url::UrlCow;

	use super::*;

	fn matches(glob: &str, url: &str) -> bool {
		Pattern::from_str(glob).unwrap().match_url(UrlCow::try_from(url).unwrap(), false)
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
