use std::path::Path;

use anyhow::{Context, Result, bail};
use tokio::process::Command;
use yazi_shared::strip_trailing_newline;

pub(super) struct Git;

pub(super) enum DiffAction {
	Add { file: String },
	Delete { file: String },
	Rename { old: String, new: String },
	Copy { old: String, new: String },
}

impl Git {
	pub(super) async fn clone(url: &str, path: &Path) -> Result<()> {
		Self::exec(|c| c.args(["clone", url]).arg(path)).await
	}

	pub(super) async fn fetch(path: &Path) -> Result<()> {
		Self::exec(|c| c.current_dir(path).arg("fetch")).await
	}

	pub(super) async fn checkout(path: &Path, rev: &str) -> Result<()> {
		Self::exec(|c| c.current_dir(path).args(["checkout", rev])).await
	}

	pub(super) async fn pull(path: &Path) -> Result<()> {
		Self::fetch(path).await?;
		Self::checkout(path, "origin/HEAD").await?;
		Ok(())
	}

	pub(super) async fn hash(path: &Path) -> Result<String> {
		let output = Command::new("git")
			.current_dir(path)
			.args(["rev-parse", "--short", "HEAD"])
			.output()
			.await
			.context("Failed to get current revision")?;

		if !output.status.success() {
			bail!("Getting revision failed: {}", output.status);
		}

		Ok(strip_trailing_newline(
			String::from_utf8(output.stdout).context("Failed to parse revision")?,
		))
	}

	pub(super) async fn diff(path: &Path, rev: &str, filter: &[&str]) -> Result<Vec<DiffAction>> {
		let mut command = Command::new("git");
		command.current_dir(path).args(["diff", "-M100", "-C100", rev, "origin/HEAD", "--name-status"]);
		if !filter.is_empty() {
			command.arg("--");
		}
		for f in filter {
			command.arg(*f);
		}
		let output = command.output().await.context("Failed to run git diff")?;

		if !output.status.success() {
			bail!("git diff failed: {}", output.status);
		}

		let stdout = String::from_utf8(output.stdout).context("Failed to parse git diff output")?;
		let mut diff_actions = Vec::new();

		for line in stdout.lines() {
			let parts: Vec<&str> = line.split_whitespace().collect();
			if parts.is_empty() {
				continue;
			}

			match parts[0] {
				"A" | "M" if parts.len() == 2 => {
					diff_actions.push(DiffAction::Add { file: parts[1].to_string() })
				}
				"D" if parts.len() == 2 => {
					diff_actions.push(DiffAction::Delete { file: parts[1].to_string() })
				}
				"R100" if parts.len() == 3 => diff_actions
					.push(DiffAction::Rename { old: parts[1].to_string(), new: parts[2].to_string() }),
				"C100" if parts.len() == 3 => diff_actions
					.push(DiffAction::Copy { old: parts[1].to_string(), new: parts[2].to_string() }),
				_ => {}
			}
		}

		Ok(diff_actions)
	}

	async fn exec(f: impl FnOnce(&mut Command) -> &mut Command) -> Result<()> {
		let status =
			f(&mut Command::new("git")).status().await.context("Failed to execute `git` command")?;

		if !status.success() {
			bail!("`git` command failed: {status}");
		}

		Ok(())
	}
}

impl DiffAction {
	pub(super) fn is_in(&self, dirs: &[&str]) -> bool {
		for dir in dirs {
			match self {
				DiffAction::Add { file } => return file.contains(dir),
				DiffAction::Delete { file } => return file.contains(dir),
				DiffAction::Rename { old: _, new } => return new.contains(dir),
				DiffAction::Copy { old: _, new } => return new.contains(dir),
			}
		}
		false
	}
}
