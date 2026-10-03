use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use serde_with::{OneOrMany, formats::PreferOne, serde_as};

#[derive(Debug, Deserialize)]
#[serde(try_from = "ExcludeShadow")]
pub struct Exclude {
	pub urn:  Vec<String>,
	pub r#in: String,
	globs:    Globs,
}

#[serde_as]
#[derive(Deserialize)]
struct ExcludeShadow {
	#[serde_as(as = "OneOrMany<_, PreferOne>")]
	urn:  Vec<String>,
	r#in: String,
}

impl TryFrom<ExcludeShadow> for Exclude {
	type Error = globset::Error;

	fn try_from(shadow: ExcludeShadow) -> Result<Self, Self::Error> {
		let mut globs = vec![];
		for pattern in &shadow.urn {
			let (negated, pattern) = split_negation(pattern);
			if pattern.contains('/') {
				globs.push((negated, Glob::new(pattern)?));
			} else {
				// A bare name matches the entry itself and anything inside it, at any depth
				globs.push((negated, Glob::new(&format!("**/{pattern}"))?));
				globs.push((negated, Glob::new(&format!("**/{pattern}/**"))?));
			}
		}

		Ok(Self { globs: Globs::build(globs)?, urn: shadow.urn, r#in: shadow.r#in })
	}
}

impl Exclude {
	pub fn matches_path(&self, path: &Path) -> Option<bool> { self.globs.matches(path) }

	pub fn matches_context(&self, context: &str) -> bool {
		let pattern = self.r#in.as_str();
		if pattern == "*" {
			return true;
		}

		if let Some(rest) = pattern.strip_prefix("**/") {
			let name = rest.strip_suffix("/**").unwrap_or(rest);
			return context == name
				|| context.ends_with(&format!("/{name}"))
				|| context.contains(&format!("/{name}/"));
		}

		if let Some(prefix) = pattern.strip_suffix("/**") {
			if context == prefix || context.starts_with(&format!("{prefix}/")) {
				return true;
			}

			// "/target/**" also matches a "target" segment anywhere in the path
			return prefix
				.strip_prefix('/')
				.filter(|s| !s.is_empty() && !s.starts_with('/'))
				.is_some_and(|seg| {
					context.contains(&format!("/{seg}/")) || context.ends_with(&format!("/{seg}"))
				});
		}

		context == pattern || context.strip_prefix(pattern).is_some_and(|s| s.starts_with('/'))
	}
}

// --- Globs
#[derive(Debug)]
pub(super) struct Globs {
	ignores:    GlobSet,
	whitelists: GlobSet,
}

impl Globs {
	pub(super) fn build(
		globs: impl IntoIterator<Item = (bool, Glob)>,
	) -> Result<Self, globset::Error> {
		let (mut ignores, mut whitelists) = (GlobSetBuilder::new(), GlobSetBuilder::new());
		for (negated, glob) in globs {
			if negated {
				whitelists.add(glob);
			} else {
				ignores.add(glob);
			}
		}

		Ok(Self { ignores: ignores.build()?, whitelists: whitelists.build()? })
	}

	pub(super) fn matches(&self, path: &Path) -> Option<bool> { self.matches_any(&[path]) }

	pub(super) fn matches_any(&self, paths: &[&Path]) -> Option<bool> {
		if paths.iter().any(|p| self.whitelists.is_match(p)) {
			Some(false)
		} else if paths.iter().any(|p| self.ignores.is_match(p)) {
			Some(true)
		} else {
			None
		}
	}
}

pub(super) fn split_negation(pattern: &str) -> (bool, &str) {
	match pattern.strip_prefix('!') {
		Some(rest) => (true, rest),
		None => (false, pattern),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn exclude(urn: &[&str], r#in: &str) -> Exclude {
		ExcludeShadow { urn: urn.iter().map(|s| s.to_string()).collect(), r#in: r#in.to_owned() }
			.try_into()
			.unwrap()
	}

	fn matches(e: &Exclude, path: &str) -> Option<bool> { e.matches_path(Path::new(path)) }

	#[test]
	fn test_bare_name() {
		let e = exclude(&[".git"], "*");
		assert_eq!(matches(&e, "/home/user/proj/.git"), Some(true));
		assert_eq!(matches(&e, "/home/user/proj/.git/config"), Some(true));
		assert_eq!(matches(&e, ".git"), Some(true));
		assert_eq!(matches(&e, "/home/user/proj/.gitignore"), None);
		assert_eq!(matches(&e, "/home/user/proj/git"), None);
	}

	#[test]
	fn test_glob() {
		let e = exclude(&[".DS_Store", "Icon?", "*.pyc"], "*");
		assert_eq!(matches(&e, "/a/.DS_Store"), Some(true));
		assert_eq!(matches(&e, "/a/Icon\r"), Some(true));
		assert_eq!(matches(&e, "/a/b/x.pyc"), Some(true));
		assert_eq!(matches(&e, "/a/Icons"), Some(true));
		assert_eq!(matches(&e, "/a/Icon"), None);
		assert_eq!(matches(&e, "/a/.visible_dot"), None);
		assert_eq!(matches(&e, "/a/normal.txt"), None);
	}

	#[test]
	fn test_path_glob() {
		let e = exclude(&["/root/**/*.pyc"], "*");
		assert_eq!(matches(&e, "/root/a/b.pyc"), Some(true));
		assert_eq!(matches(&e, "/other/a/b.pyc"), None);

		// An explicit "**/" prefix is not expanded to the entries inside
		let e = exclude(&["**/.git"], "*");
		assert_eq!(matches(&e, "/a/.git"), Some(true));
		assert_eq!(matches(&e, "/a/.git/config"), None);
	}

	#[test]
	fn test_negation() {
		let e = exclude(&["*.json", "!package.json"], "*");
		assert_eq!(matches(&e, "/a/data.json"), Some(true));
		assert_eq!(matches(&e, "/a/package.json"), Some(false));
		assert_eq!(matches(&e, "/a/readme.md"), None);

		let e = exclude(&["!target"], "*");
		assert_eq!(matches(&e, "/a/target"), Some(false));
		assert_eq!(matches(&e, "/a/target/debug"), Some(false));
	}

	#[test]
	fn test_context() {
		let all = exclude(&["x"], "*");
		assert!(all.matches_context("/any/where"));
		assert!(all.matches_context("sftp://host//x"));

		let prefix = exclude(&["x"], "/code/**");
		assert!(prefix.matches_context("/code"));
		assert!(prefix.matches_context("/code/proj"));
		assert!(prefix.matches_context("/home/user/code/proj"));
		assert!(!prefix.matches_context("/codex"));
		assert!(!prefix.matches_context("/other"));

		let segment = exclude(&["x"], "**/target");
		assert!(segment.matches_context("/proj/target"));
		assert!(segment.matches_context("/proj/target/debug"));
		assert!(!segment.matches_context("/proj/xtarget"));

		let exact = exclude(&["x"], "/code");
		assert!(exact.matches_context("/code"));
		assert!(exact.matches_context("/code/proj"));
		assert!(!exact.matches_context("/codex"));

		let sftp = exclude(&["x"], "sftp://**");
		assert!(sftp.matches_context("sftp://host//home/user"));
		assert!(!sftp.matches_context("/home/user"));

		let search = exclude(&["x"], "search://**");
		assert!(search.matches_context("search://**"));
		assert!(!search.matches_context("/home/user"));
	}

	#[test]
	fn test_deserialize() {
		#[derive(Deserialize)]
		struct T {
			excludes: Vec<Exclude>,
		}

		let t: T = toml::from_str(
			r#"excludes = [ { urn = ".DS_Store", in = "*" }, { urn = [ "a", "!b" ], in = "/c/**" } ]"#,
		)
		.unwrap();
		assert_eq!(t.excludes[0].urn, [".DS_Store"]);
		assert_eq!(t.excludes[1].urn, ["a", "!b"]);
		assert_eq!(t.excludes[1].r#in, "/c/**");

		assert!(toml::from_str::<T>(r#"excludes = [ { urn = "a[", in = "*" } ]"#).is_err());
	}
}
