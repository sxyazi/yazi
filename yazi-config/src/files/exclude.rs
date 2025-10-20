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
				let glob = Glob::new(negated)?;
				whitelist_builder.add(glob);
			} else {
				// Regular pattern - add to ignore list
				let glob = Glob::new(pattern)?;
				ignore_builder.add(glob);
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

		// For absolute paths, try both the full path and relative components
		// This helps patterns like **/.git/** match /home/user/project/.git
		let paths_to_check: Vec<&Path> = if path.is_absolute() {
			// Also check each component as if it's relative
			// This allows **/.git/** to match /home/user/.git/config
			let mut paths = vec![path];

			// Check if any path component matches by checking relative sub-paths
			// For /home/user/.git/config, we want to match against .git/config too
			if let Some(components) = path.to_str() {
				for (i, _) in components.match_indices('/').skip(1) {
					if let Some(subpath) = components.get(i + 1..) {
						paths.push(Path::new(subpath));
					}
				}
			}
			paths
		} else {
			vec![path]
		};

		// Check whitelist first (negation takes precedence)
		for p in &paths_to_check {
			if compiled.whitelists.is_match(p) {
				return Some(false); // Explicitly NOT ignored
			}
		}

		// Check ignore patterns
		for p in &paths_to_check {
			if compiled.ignores.is_match(p) {
				return Some(true); // Should be ignored
			}
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
			let pattern = if let Some(p) = pattern.strip_suffix("/**") {
				p
			} else {
				pattern
			};

			// Check if path ends with the pattern (e.g., /home/user/project/target matches **/target)
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
