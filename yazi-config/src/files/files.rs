use std::path::Path;

use serde::Deserialize;
use yazi_codegen::DeserializeOver2;

use super::Exclude;

/// Configuration for file filtering
#[derive(Debug, Deserialize, DeserializeOver2, Default)]
pub struct Files {
	/// List of exclude rules with context-specific patterns
	#[serde(default)]
	pub excludes: Vec<Exclude>,
}

impl Files {
	/// Compile all glob patterns in exclude rules
	pub fn compile(&mut self) -> Result<(), String> {
		for exclude in &mut self.excludes {
			exclude.compile().map_err(|e| format!("Failed to compile glob pattern: {}", e))?;
		}
		Ok(())
	}

	/// Get all exclude patterns that apply to a given context
	pub fn excludes_for_context(&self, context: &str) -> Vec<String> {
		self
			.excludes
			.iter()
			.filter(|e| e.matches_context(context))
			.flat_map(|e| e.urn.iter().cloned())
			.collect()
	}

	/// Check if a path should be excluded based on compiled patterns for a given
	/// context Returns Some(true) if should be ignored, Some(false) if
	/// whitelisted, None if no match
	pub fn matches_path(&self, path: &Path, context: &str) -> Option<bool> {
		// Process rules in order, last match wins
		let mut result = None;

		for exclude in &self.excludes {
			if exclude.matches_context(context)
				&& let Some(should_ignore) = exclude.matches_path(path)
			{
				result = Some(should_ignore);
			}
		}

		result
	}
}
