use std::path::Path;

use anyhow::{Context, Result, bail};
use tokio::process::Command;
use yazi_shared::strip_trailing_newline;

pub(super) struct Git;

impl Git {
	pub(super) async fn clone(url: &str, path: &Path) -> Result<()> {
		Self::exec(|c| c.args(["clone", url]).arg(path)).await
	}

	pub(super) async fn fetch(path: &Path) -> Result<()> {
		Self::exec(|c| c.arg("fetch").current_dir(path)).await
	}

	pub(super) async fn checkout(path: &Path, rev: &str) -> Result<()> {
		Self::exec(|c| c.args(["checkout", rev]).current_dir(path)).await
	}

	pub(super) async fn pull(path: &Path) -> Result<()> {
		Self::fetch(path).await?;
		Self::checkout(path, "origin/HEAD").await?;
		Ok(())
	}

	pub(super) async fn revision(path: &Path) -> Result<String> {
		let output = Command::new("git")
			.args(["rev-parse", "--short", "HEAD"])
			.current_dir(path)
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
		let status = f(Command::new("git").args(["-c", "advice.detachedHead=false"]))
			.status()
			.await
			.context("Failed to execute `git` command")?;

		if !status.success() {
			bail!("`git` command failed: {status}");
		}

		Ok(())
	}
}
