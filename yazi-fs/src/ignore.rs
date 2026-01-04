use std::{path::Path, sync::Arc};

use yazi_shared::url::AsUrl;

/// Filter for ignoring files based on custom exclude patterns.
///
/// Exclude patterns can be context-specific and support negation using the `!`
/// prefix for whitelisting.
#[derive(Clone)]
pub struct IgnoreFilter {
	/// Custom glob-based matcher function for pattern matching
	/// Returns Some(true) if should be ignored, Some(false) if whitelisted, None
	/// if no match
	glob_matcher: Option<Arc<dyn Fn(&Path) -> Option<bool> + Send + Sync>>,
}

impl std::fmt::Debug for IgnoreFilter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("IgnoreFilter")
			.field("glob_matcher", &self.glob_matcher.as_ref().map(|_| "Some(...)"))
			.finish()
	}
}

impl IgnoreFilter {
	/// Creates a new `IgnoreFilter` from exclude patterns.
	///
	/// # Arguments
	///
	/// * `glob_matcher` - Custom glob-based matcher function for advanced pattern
	///   matching
	///
	/// # Returns
	///
	/// `Some(IgnoreFilter)` if glob_matcher is provided, `None` otherwise.
	pub fn from_patterns(
		glob_matcher: Option<Arc<dyn Fn(&Path) -> Option<bool> + Send + Sync>>,
	) -> Option<Self> {
		if glob_matcher.is_none() {
			return None;
		}

		Some(Self { glob_matcher })
	}

	/// Checks if a file should be ignored based on its URL.
	///
	/// # Arguments
	///
	/// * `url` - URL of the file to check
	///
	/// # Returns
	///
	/// `true` if the file should be ignored, `false` otherwise.
	///
	/// # Matching Logic
	///
	/// Uses the glob matcher to check if the path should be ignored.
	/// Returns true if matched as ignore, false if whitelisted or no match.
	pub fn matches_url(&self, url: impl AsUrl) -> bool {
		let url = url.as_url();
		let Ok(path) = url.loc().as_os() else { return false };

		// Check glob matcher
		if let Some(ref matcher) = self.glob_matcher {
			if let Some(should_ignore) = matcher(path) {
				return should_ignore;
			}
		}

		false
	}
}
