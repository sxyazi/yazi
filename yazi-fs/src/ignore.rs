use std::{collections::HashSet, path::{Path, PathBuf}};

use yazi_shared::url::AsUrl;

/// Filter for ignoring files based on git ignore rules and custom exclude
/// patterns.
///
/// This filter combines git's native ignore functionality with custom exclude
/// patterns that follow gitignore syntax. Exclude patterns can be
/// context-specific and take precedence over git's ignore rules using the `!`
/// prefix for negation.
#[derive(Clone, Debug)]
pub struct IgnoreFilter {
	/// Set of paths ignored by git (from git status)
	ignored_paths: HashSet<PathBuf>,
	/// Custom gitignore matcher for exclude patterns
	gitignore:     Option<ignore::gitignore::Gitignore>,
}

impl IgnoreFilter {
	/// Creates a new `IgnoreFilter` by checking git ignore status for the given
	/// directory.
	///
	/// # Arguments
	///
	/// * `dir` - Directory to check for git ignore status
	/// * `exclude_patterns` - Custom gitignore patterns that override git's rules
	/// * `use_git` - Whether to integrate with git ignore rules
	///
	/// # Returns
	///
	/// `Some(IgnoreFilter)` if git repository is found (when use_git=true) or
	/// exclude patterns exist, `None` if no git repository and no exclude
	/// patterns.
	///
	/// # Behavior
	///
	/// - Discovers the git repository containing `dir` (if use_git=true)
	/// - Collects all ignored paths from git status
	/// - Builds a custom gitignore matcher from exclude patterns (relative to
	///   repo root)
	/// - Exclude patterns are checked first and take precedence over git ignore
	///   rules
	/// - Negation patterns (starting with `!`) can whitelist git-ignored files
	pub fn from_dir(
		dir: impl AsRef<Path>,
		exclude_patterns: &[String],
		use_git: bool,
	) -> Option<Self> {
		let dir = dir.as_ref();

		let (workdir, ignored_paths) = if use_git {
			// Try to open the git repository for this directory
			let repo = git2::Repository::discover(dir).ok()?;

			// Get the workdir (root of the git repository)
			let workdir = repo.workdir()?.to_path_buf();

			// Get git statuses for the repository
			let statuses = match repo.statuses(None) {
				Ok(s) => s,
				Err(_) => return None,
			};

			// Build a set of ALL ignored paths from git status
			let mut ignored_paths = HashSet::new();

			// Manually add .git directory as ignored (like eza does)
			ignored_paths.insert(workdir.join(".git"));

			// Add all ignored files from git status
			for status in statuses.iter() {
				if status.status() == git2::Status::IGNORED
					&& let Some(path) = status.path()
				{
					ignored_paths.insert(workdir.join(path));
				}
			}

			(workdir, ignored_paths)
		} else {
			// No git integration, use current directory as base
			(dir.to_path_buf(), HashSet::new())
		};

		// Build custom gitignore from exclude patterns if provided
		let gitignore = if !exclude_patterns.is_empty() {
			let mut builder = ignore::gitignore::GitignoreBuilder::new(&workdir);

			// Add each exclude pattern
			for pattern in exclude_patterns {
				let _ = builder.add_line(None, pattern);
			}

			builder.build().ok()
		} else {
			None
		};

		if ignored_paths.is_empty()
			|| (ignored_paths.len() == 1 && ignored_paths.contains(&workdir.join(".git")))
		{
			// If we have no git-ignored paths but we have exclude patterns, still create
			// the filter
			if gitignore.is_some() {
				return Some(Self { ignored_paths, gitignore });
			}
			return None;
		}

		// Store ALL ignored paths, not just the ones in the current directory
		// This way, when files are loaded later, we can check them against the full set
		Some(Self { ignored_paths, gitignore })
	}

	/// Creates a new `IgnoreFilter` from only exclude patterns without git
	/// integration.
	///
	/// # Arguments
	///
	/// * `dir` - Base directory for pattern matching
	/// * `patterns` - Gitignore-style patterns to apply
	///
	/// # Returns
	///
	/// `Some(IgnoreFilter)` if patterns are provided, `None` if patterns is
	/// empty.
	///
	/// This is useful when `gitignores = false` but custom exclude patterns
	/// are still desired.
	pub fn from_patterns(dir: impl AsRef<Path>, patterns: &[String]) -> Option<Self> {
		if patterns.is_empty() {
			return None;
		}

		let dir = dir.as_ref();
		let mut builder = ignore::gitignore::GitignoreBuilder::new(dir);

		// Add each pattern
		for pattern in patterns {
			let _ = builder.add_line(None, pattern);
		}

		let gitignore = builder.build().ok()?;

		Some(Self { ignored_paths: HashSet::new(), gitignore: Some(gitignore) })
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
	/// 1. Check override patterns first:
	///    - If matched as whitelist (negation `!`), return `false` (not ignored)
	///    - If matched as ignore, return `true` (ignored)
	/// 2. Check if path is directly in git's ignored set
	/// 3. Check if any parent directory is ignored by git
	///    - If parent is ignored, also check if override patterns whitelist it
	///    - If parent is whitelisted, children are not ignored
	///
	/// This ensures override patterns take precedence and negation patterns work
	/// correctly even when navigating inside ignored directories.
	pub fn matches_url(&self, url: impl AsUrl) -> bool {
		let url = url.as_url();
		let path = url.loc.as_path();

		// First check if override patterns apply (they can negate ignores)
		// Override patterns take absolute priority
		if let Some(ref gitignore) = self.gitignore {
			let matched = gitignore.matched(path, path.is_dir());
			match matched {
				ignore::Match::None => {
					// No override pattern matched, fall through to git ignore check
				}
				ignore::Match::Ignore(_) => {
					// Override pattern says to ignore
					return true;
				}
				ignore::Match::Whitelist(_) => {
					// Override pattern says NOT to ignore (negation pattern like !target/)
					// This takes precedence over git ignore
					return false;
				}
			}
		}

		// Check exact match first
		if self.ignored_paths.contains(path) {
			return true;
		}

		// Check if any parent directory is ignored
		// BUT also check if parent is whitelisted by override patterns
		let mut current = path;
		while let Some(parent) = current.parent() {
			if self.ignored_paths.contains(parent) {
				// Parent is ignored by git, but check if override patterns whitelist it
				if let Some(ref gitignore) = self.gitignore {
					let matched = gitignore.matched(parent, true); // parent is always a directory
					if matches!(matched, ignore::Match::Whitelist(_)) {
						// Parent is whitelisted, so children should not be ignored
						return false;
					}
				}
				return true;
			}
			current = parent;
		}

		false
	}

	/// Check if a path should be ignored
	pub fn matches_path(&self, path: &Path) -> bool { self.ignored_paths.contains(path) }
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_git_repo_discovery() {
		// This test assumes we're running in the yazi git repo
		let current_dir = std::env::current_dir().unwrap();
		assert!(git2::Repository::discover(&current_dir).is_ok());
	}
}
