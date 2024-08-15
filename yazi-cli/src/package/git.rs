use std::path::Path;

use anyhow::{bail, Context, Result};
use tokio::process::Command;
use yazi_shared::strip_trailing_newline;

pub(super) struct Git;

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

	async fn exec(f: impl FnOnce(&mut Command) -> &mut Command) -> Result<()> {
		let status =
			f(&mut Command::new("git")).status().await.context("Failed to execute `git` command")?;

		if !status.success() {
			bail!("`git` command failed: {status}");
		}

		Ok(())
	}
}
