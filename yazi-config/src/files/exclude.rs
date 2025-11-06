use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;

/// Represents a single exclude rule with patterns and context
#[derive(Debug, Clone, Deserialize)]
pub struct Exclude {
	/// Pattern(s) to match files/directories against
	/// Can be a single glob pattern string or an array of glob patterns
	/// Patterns starting with '!' negate (whitelist) previously matched patterns
	#[serde(deserialize_with = "deserialize_urn")]
	pub urn: Vec<String>,

	/// Context where this exclude rule applies
	/// Supports glob patterns like "/code/**", "sftp://**", "search://**", or "*" for all
	pub r#in: String,

	#[serde(skip)]
	compiled: Option<CompiledPatterns>,
}

#[derive(Debug, Clone)]
struct CompiledPatterns {
	/// Regular patterns (to ignore)
	ignores:    GlobSet,
	/// Negated patterns (to whitelist/un-ignore)
	whitelists: GlobSet,
}

fn deserialize_urn<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	#[derive(Deserialize)]
	#[serde(untagged)]
	enum UrnOrUrns {
		Single(String),
		Multiple(Vec<String>),
	}

	match UrnOrUrns::deserialize(deserializer)? {
		UrnOrUrns::Single(s) => Ok(vec![s]),
		UrnOrUrns::Multiple(v) => Ok(v),
	}
}

impl Exclude {
	/// Compile the glob patterns into GlobSets for efficient matching
	pub fn compile(&mut self) -> Result<(), globset::Error> {
		let mut ignore_builder = GlobSetBuilder::new();
		let mut whitelist_builder = GlobSetBuilder::new();

		for pattern in &self.urn {
			if let Some(negated) = pattern.strip_prefix('!') {
				// Negation pattern - add to whitelist
				// Transform simple filename patterns to match anywhere in the tree
				let transformed = if negated.contains('/') || negated.starts_with("**/") {
					negated.to_string()
				} else {
					// Match the item itself and everything inside it
					format!("**/{}", negated)
				};
				let glob = Glob::new(&transformed)?;
				whitelist_builder.add(glob);
				// Also match everything inside directories with this name
				if !negated.contains('/') && !negated.starts_with("**/") {
					let glob_inner = Glob::new(&format!("**/{}/**", negated))?;
					whitelist_builder.add(glob_inner);
				}
			} else {
				// Regular pattern - add to ignore list
				// Transform simple filename patterns to match anywhere in the tree
				let transformed = if pattern.contains('/') || pattern.starts_with("**/") {
					pattern.to_string()
				} else {
					// Match the item itself: **/.git
					format!("**/{}", pattern)
				};
				let glob = Glob::new(&transformed)?;
				ignore_builder.add(glob);
				// Also match everything inside directories with this name: **/.git/**
				if !pattern.contains('/') && !pattern.starts_with("**/") {
					let glob_inner = Glob::new(&format!("**/{}/**", pattern))?;
					ignore_builder.add(glob_inner);
				}
			}
		}

		self.compiled = Some(CompiledPatterns {
			ignores:    ignore_builder.build()?,
			whitelists: whitelist_builder.build()?,
		});

		Ok(())
	}

	/// Check if a path matches this exclude rule
	/// Returns Some(true) if path should be ignored, Some(false) if whitelisted,
	/// None if no match
	pub fn matches_path(&self, path: &Path) -> Option<bool> {
		let compiled = self.compiled.as_ref()?;

		// Check whitelist first (negation takes precedence)
		if compiled.whitelists.is_match(path) {
			return Some(false); // Explicitly NOT ignored
		}

		// Check ignore patterns
		if compiled.ignores.is_match(path) {
			return Some(true); // Should be ignored
		}

		None // No match
	}

	/// Check if this exclude rule applies to the given path context
	pub fn matches_context(&self, path: &str) -> bool {
		// Wildcard matches everything
		if self.r#in == "*" {
			return true;
		}

		// Handle patterns starting with **/ (e.g., **/target or **/target/**)
		if self.r#in.starts_with("**/") {
			let pattern = &self.r#in[3..]; // Remove leading "**/", e.g., "target" or "target/**"

			// Strip trailing /** if present
			let pattern = if let Some(p) = pattern.strip_suffix("/**") { p } else { pattern };

			// Check if path ends with the pattern (e.g., /home/user/project/target matches
			// **/target)
			if path.ends_with(&format!("/{}", pattern)) || path.ends_with(pattern) {
				return true;
			}

			// Check if path contains the pattern as a directory segment
			if path.contains(&format!("/{}/", pattern)) {
				return true;
			}

			return false;
		}

		// Handle glob patterns with wildcard
		if self.r#in.ends_with("/**") {
			let prefix = &self.r#in[..self.r#in.len() - 3];

			// Check if path starts with prefix (absolute path match)
			if path.starts_with(prefix) {
				return true;
			}

			// Check if path contains the pattern anywhere (for relative patterns like
			// "/target/**") This allows "/target/**" to match
			// "/home/user/project/target/debug"
			if prefix.starts_with('/') && !prefix.starts_with("//") {
				// Single leading slash means relative pattern - check if path contains this
				// segment
				let pattern = &prefix[1..]; // Remove leading slash
				if path.contains(&format!("/{}/", pattern)) || path.ends_with(&format!("/{}", pattern)) {
					return true;
				}
			}

			return false;
		}

		// Exact match or prefix match for non-wildcard patterns
		path == self.r#in || path.starts_with(&format!("{}/", self.r#in))
	}
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use super::*;

	fn create_exclude(in_pattern: &str, urn_patterns: Vec<&str>) -> Exclude {
		let mut exclude = Exclude {
			r#in:     in_pattern.to_string(),
			urn:      urn_patterns.into_iter().map(String::from).collect(),
			compiled: None,
		};
		exclude.compile().unwrap();
		exclude
	}

	#[test]
	fn test_context_matching_with_dots() {
		let exclude = create_exclude("*", vec![".git"]);

		// Test various path contexts - all should match "*"
		let test_cases = vec![
			("/home/user/projects/yazi/yazi", true),
			("/home/user/projects/yazi/yazi-rs.github.io", true),
			("/home/user/projects/yazi/command-palette.yazi", true),
			("/home/user/projects/yazi/gitignore.yazi", true),
		];

		for (context, should_match) in test_cases {
			let matches = exclude.matches_context(context);
			assert_eq!(
				matches, should_match,
				"Context matching failed for {}: expected {}, got {}",
				context, should_match, matches
			);
		}
	}

	#[test]
	fn test_simple_pattern_behavior() {
		// Test how globset matches simple patterns like ".git"
		use globset::Glob;

		let patterns_and_paths = vec![
			// Pattern ".git" gets transformed to "**/.git" in our compile() method
			// So let's test the transformed version
			("**/.git", "/home/user/.git", true),
			("**/.git", "/home/user/proj/.git", true),
			("**/.git", "/home/user/command-palette.yazi/.git", true),
			("**/.git", ".git", true),
			// **/.git/** matches files INSIDE .git, not the .git directory itself
			("**/.git/**", "/home/user/.git", false),
			("**/.git/**", "/home/user/.git/config", true),
		];

		for (pattern, path, expected) in patterns_and_paths {
			let glob = Glob::new(pattern).unwrap().compile_matcher();
			let matches = glob.is_match(Path::new(path));
			assert_eq!(
				matches, expected,
				"Pattern '{}' with path '{}': expected {}, got {}",
				pattern, path, expected, matches
			);
		}
	}

	#[test]
	fn test_path_matching_with_git() {
		let exclude = create_exclude("*", vec![".git"]);

		// Test various .git paths
		// Pattern ".git" is transformed to "**/.git" which matches any component named
		// .git
		let test_paths = vec![
			("/home/user/projects/yazi/yazi/.git", true),
			("/home/user/projects/yazi/yazi-rs.github.io/.git", true),
			("/home/user/projects/yazi/command-palette.yazi/.git", true),
			("/home/user/projects/yazi/gitignore.yazi/.git", true),
			// Files inside .git now also match "**/.git" since .git is the component name
			("/home/user/projects/yazi/yazi/.git/config", true),
			("/home/user/projects/yazi/command-palette.yazi/.git/config", true),
		];

		for (path_str, should_match) in test_paths {
			let path = Path::new(path_str);
			let result = exclude.matches_path(path);
			// None means no match, which is equivalent to false (not ignored)
			let actual = result.unwrap_or(false);
			assert_eq!(
				actual, should_match,
				"Path matching failed for {}: expected {}, got {}",
				path_str, should_match, actual
			);
		}
	}

	#[test]
	fn test_glob_pattern_matching() {
		// User provides "**/.git" explicitly - matches only things NAMED .git
		let exclude = create_exclude("*", vec!["**/.git"]);

		let test_paths = vec![
			// Should match .git directory itself
			("/home/user/projects/yazi/yazi/.git", true),
			("/home/user/projects/yazi/command-palette.yazi/.git", true),
			// Should NOT match files inside .git when using **/.git alone
			("/home/user/projects/yazi/yazi/.git/config", false),
		];

		for (path_str, should_match) in test_paths {
			let path = Path::new(path_str);
			let result = exclude.matches_path(path);
			let actual = result.unwrap_or(false);
			assert_eq!(
				actual, should_match,
				"Glob pattern matching failed for {}: expected {}, got {}",
				path_str, should_match, actual
			);
		}
	}

	#[test]
	fn test_glob_pattern_with_trailing_slash() {
		// Pattern **/.git/** means match everything inside any .git directory
		let exclude = create_exclude("*", vec!["**/.git/**"]);

		let test_paths = vec![
			// Should NOT match .git directory itself with /**
			("/home/user/projects/yazi/yazi/.git", false),
			// Should match all files inside .git
			("/home/user/projects/yazi/yazi/.git/config", true),
			("/home/user/projects/yazi/command-palette.yazi/.git", false),
			("/home/user/projects/yazi/command-palette.yazi/.git/config", true),
		];

		for (path_str, should_match) in test_paths {
			let path = Path::new(path_str);
			let result = exclude.matches_path(path);
			let actual = result.unwrap_or(false);
			assert_eq!(
				actual, should_match,
				"Glob pattern with /** matching failed for {}: expected {}, got {}",
				path_str, should_match, actual
			);
		}
	}
}
