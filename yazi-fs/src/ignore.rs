use std::{collections::HashSet, path::{Path, PathBuf}};

use yazi_shared::url::AsUrl;

#[derive(Clone, Debug)]
pub struct IgnoreFilter {
	ignored_paths: HashSet<PathBuf>,
	gitignore:     Option<ignore::gitignore::Gitignore>,
}

impl IgnoreFilter {
	/// Create a new IgnoreFilter by checking git ignore status for files in the
	/// given directory
	pub fn from_dir(dir: impl AsRef<Path>, override_patterns: &[String]) -> Option<Self> {
		let dir = dir.as_ref();

		// Try to open the git repository for this directory
		let repo = git2::Repository::discover(dir).ok()?;

		// Get the workdir (root of the git repository)
		let workdir = repo.workdir()?;

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
			if status.status() == git2::Status::IGNORED {
				if let Some(path) = status.path() {
					ignored_paths.insert(workdir.join(path));
				}
			}
		}

		// Build custom gitignore from override patterns if provided
		let gitignore = if !override_patterns.is_empty() {
			let mut builder = ignore::gitignore::GitignoreBuilder::new(workdir);

			// Add each override pattern
			for pattern in override_patterns {
				let _ = builder.add_line(None, pattern);
			}

			builder.build().ok()
		} else {
			None
		};

		if ignored_paths.is_empty()
			|| (ignored_paths.len() == 1 && ignored_paths.contains(&workdir.join(".git")))
		{
			// If we have no git-ignored paths but we have override patterns, still create
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

	/// Create a new IgnoreFilter from only override patterns (no git integration)
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

	/// Check if a file should be ignored based on its URL
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
	use super::*;

	#[test]
	fn test_git_repo_discovery() {
		// This test assumes we're running in the yazi git repo
		let current_dir = std::env::current_dir().unwrap();
		assert!(git2::Repository::discover(&current_dir).is_ok());
	}
}
