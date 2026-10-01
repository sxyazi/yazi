use std::path::Path;

use globset::Glob;
use serde::Deserialize;
use yazi_codegen::{DeserializeOver, DeserializeOver2};
use yazi_fs::IgnoreFilter;
use yazi_macro::warn;
use yazi_shared::url::Url;

use super::{Exclude, Globs, split_negation};

#[derive(Debug, Deserialize, DeserializeOver, DeserializeOver2)]
pub struct Files {
	pub excludes: Vec<Exclude>,
}

impl Files {
	/// Builds the filter for the entries of `dir`, from the rules whose `in`
	/// matches it, plus `extra` patterns from plugins.
	pub fn ignore_filter(&'static self, dir: Url, extra: &[String]) -> Option<IgnoreFilter> {
		if self.excludes.is_empty() && extra.is_empty() {
			return None;
		}

		let context = if dir.is_view() { "search://**".to_owned() } else { dir.to_string() };
		let rules = self.rules(&context);
		let extra = Self::extra(extra, &rules);
		if rules.is_empty() && extra.is_none() {
			return None;
		}

		let filter = IgnoreFilter::new(move |path| {
			matches(&rules, path).or_else(|| extra.as_ref()?.matches_any(&suffixes(path))) == Some(true)
		});

		// Inside an excluded directory, show everything
		if filter.matches_url(dir) { None } else { Some(filter) }
	}

	fn rules(&self, context: &str) -> Vec<&Exclude> {
		self.excludes.iter().filter(|e| e.matches_context(context)).collect()
	}

	// Plugin patterns are matched as written, against the path and each of its
	// trailing parts, so that "target" matches "/proj/target".
	fn extra(extra: &[String], rules: &[&Exclude]) -> Option<Globs> {
		if extra.is_empty() {
			return None;
		}

		let patterns = extra.iter().chain(rules.iter().flat_map(|e| &e.urn));
		Globs::build(patterns.filter_map(|p| {
			let (negated, p) = split_negation(p);
			Glob::new(p)
				.inspect_err(|e| warn!("Invalid exclude pattern {p:?}: {e}"))
				.ok()
				.map(|g| (negated, g))
		}))
		.inspect_err(|e| warn!("Failed to build exclude patterns: {e}"))
		.ok()
	}
}

// The last matching rule wins
fn matches(rules: &[&Exclude], path: &Path) -> Option<bool> {
	rules.iter().rev().find_map(|e| e.matches_path(path))
}

fn suffixes(path: &Path) -> Vec<&Path> {
	let mut paths = vec![path];
	if let Some(s) = path.to_str() {
		paths.extend(s.match_indices('/').skip(1).map(|(i, _)| Path::new(&s[i + 1..])));
	}
	paths
}

#[cfg(test)]
mod tests {
	use super::*;

	fn files(toml: &str) -> Files { toml::from_str(toml).unwrap() }

	fn decide(files: &Files, context: &str, path: &str) -> Option<bool> {
		matches(&files.rules(context), Path::new(path))
	}

	#[test]
	fn test_fallback() {
		let f = files(
			r#"excludes = [
				{ urn = ".DS_Store", in = "*" },
				{ urn = "node_modules", in = "/code/**" },
			]"#,
		);

		assert_eq!(decide(&f, "/code/app", "/code/app/node_modules"), Some(true));
		assert_eq!(decide(&f, "/code/app", "/code/app/.DS_Store"), Some(true));
		assert_eq!(decide(&f, "/home", "/home/node_modules"), None);
		assert_eq!(decide(&f, "/home", "/home/.DS_Store"), Some(true));
	}

	#[test]
	fn test_last_match_wins() {
		let f = files(
			r#"excludes = [
				{ urn = "target", in = "*" },
				{ urn = "!target", in = "/keep/**" },
			]"#,
		);

		assert_eq!(decide(&f, "/proj", "/proj/target"), Some(true));
		assert_eq!(decide(&f, "/keep/proj", "/keep/proj/target"), Some(false));
	}

	#[test]
	fn test_suffixes() {
		let paths: Vec<_> = suffixes(Path::new("/a/b/c")).into_iter().map(Path::to_owned).collect();
		assert_eq!(paths, [Path::new("/a/b/c"), Path::new("b/c"), Path::new("c")]);
	}

	#[test]
	fn test_extra() {
		let f = files(r#"excludes = [ { urn = "!keep", in = "*" } ]"#);
		let rules = f.rules("/proj");
		let extra = Files::extra(&["target".to_owned(), "*.log".to_owned()], &rules).unwrap();

		let check = |path: &str| extra.matches_any(&suffixes(Path::new(path)));
		assert_eq!(check("/proj/target"), Some(true));
		assert_eq!(check("/proj/a.log"), Some(true));
		assert_eq!(check("/proj/keep"), Some(false));
		assert_eq!(check("/proj/src"), None);
	}
}
