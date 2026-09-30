use std::{fmt::Debug, str::FromStr};

use anyhow::{Result, bail};
use globset::{Candidate, GlobBuilder};
use mlua::{IntoLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use serde_with::DeserializeFromStr;
use strum::EnumIs;
use yazi_shared::{KebabCasedKey, auth::Auth, url::AsUrl};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::Mixable;

#[derive(Clone, DeserializeFromStr)]
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
		} else if !self.scheme.matches(url.auth()) {
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

	pub(crate) fn match_mime(&self, mime: impl AsRef<str>) -> bool {
		self.is_star || (!mime.as_ref().is_empty() && self.inner.is_match(mime.as_ref()))
	}
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

impl Mixable for Pattern {
	fn any_file(&self) -> bool { self.is_star && !self.is_dir && self.scheme.is_any() }

	fn any_dir(&self) -> bool { self.is_star && self.is_dir && self.scheme.is_any() }
}

impl UserData for Pattern {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("scheme", |lua, me| me.scheme.into_lua(lua));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
			lua.create_string(me.inner.glob().glob())
		});
	}
}

// --- Scheme
#[derive(Clone, Debug, EnumIs)]
enum PatternScheme {
	Any,
	Local,
	Remote,

	Custom(KebabCasedKey),
}

impl PatternScheme {
	fn parse(s: &str) -> Result<(Self, usize)> {
		let Some((s, _)) = s.split_once("://") else {
			return Ok((Self::Any, 0));
		};

		let scheme = match s {
			"*" => Self::Any,
			"local" => Self::Local,
			"remote" => Self::Remote,

			"" => bail!("Invalid URL pattern: scheme is empty"),
			other if let Some(k) = KebabCasedKey::new(other) => Self::Custom(k),
			other => bail!("scheme must be 1-20 characters in kebab-case, got: {other}"),
		};

		Ok((scheme, s.len() + 3))
	}

	#[inline]
	fn matches(&self, auth: &Auth) -> bool {
		match self {
			Self::Any => true,
			Self::Local => auth.is_local(),
			Self::Remote => auth.is_remote(),
			Self::Custom(name) => auth.scheme == name,
		}
	}
}

impl IntoLua for &PatternScheme {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			PatternScheme::Any => "*",
			PatternScheme::Local => "local",
			PatternScheme::Remote => "remote",
			PatternScheme::Custom(name) => name,
		}
		.into_lua(lua)
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
		yazi_shared::init_tests();

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
		yazi_shared::init_tests();

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
