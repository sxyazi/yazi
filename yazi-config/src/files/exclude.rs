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
	#[serde(rename = "in")]
	pub context: String,

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
		if self.context == "*" {
			return true;
		}

		// Handle glob patterns
		if self.context.ends_with("/**") {
			let prefix = &self.context[..self.context.len() - 3];
			path.starts_with(prefix)
		} else {
			path == self.context || path.starts_with(&format!("{}/", self.context))
		}
	}
}
