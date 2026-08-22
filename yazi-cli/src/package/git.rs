use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use tokio::{fs, process::Command};
use yazi_shared::strip_trailing_newline;
use yazi_shim::wtf8::{FromWtf8, FromWtf8Vec};

pub(super) struct Git;

impl Git {
	pub(super) async fn clone(url: &str, path: &Path) -> Result<()> {
		Self::exec(|c| c.args(["clone", url]).arg(path)).await?;
		Self::materialize(path).await
	}

	pub(super) async fn fetch(path: &Path) -> Result<()> {
		Self::exec(|c| c.arg("fetch").current_dir(path)).await
	}

	pub(super) async fn checkout(path: &Path, rev: &str) -> Result<()> {
		Self::exec(|c| c.args(["checkout", rev, "--force"]).current_dir(path)).await?;
		Self::materialize(path).await
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

	async fn materialize(path: &Path) -> Result<()> {
		let path = fs::canonicalize(path).await.context("Failed to resolve Git repository")?;
		let output = Command::new("git")
			.args(["ls-files", "--stage", "-z"])
			.current_dir(&path)
			.output()
			.await
			.context("Failed to list Git files")?;
		if !output.status.success() {
			bail!("Listing Git files failed: {}", output.status);
		}

		for ent in output.stdout.split(|&c| c == 0).filter(|b| !b.is_empty()) {
			if !ent.starts_with(b"120000 ") {
				continue;
			}

			let Some(tab) = ent.iter().position(|&b| b == b'\t') else { continue };
			let link = path.join(
				Path::from_wtf8(&ent[tab + 1..]).context("Git path cannot be represented by the OS")?,
			);

			let original = PathBuf::from_wtf8_vec(fs::read(&link).await?)
				.context("Git symlink origin cannot be represented by the OS")?;
			let original = fs::canonicalize(link.parent().unwrap_or(&path).join(original))
				.await
				.with_context(|| format!("failed to resolve Git symlink target `{}`", link.display()))?;

			if !original.starts_with(&path) {
				bail!("Git symlink target escapes repository: `{}`", link.display());
			}

			fs::copy(original, &link)
				.await
				.with_context(|| format!("failed to materialize `{}`", link.display()))?;
		}

		Ok(())
	}

	async fn exec(f: impl FnOnce(&mut Command) -> &mut Command) -> Result<()> {
		let status = f(Command::new("git").args([
			"-c",
			"core.eol=lf",
			"-c",
			"core.autocrlf=false",
			"-c",
			"core.symlinks=false",
			"-c",
			"clone.defaultRemoteName=origin",
			"-c",
			"checkout.defaultRemote=origin",
			"-c",
			"advice.detachedHead=false",
		]))
		.status()
		.await
		.context("Failed to execute `git` command")?;

		if !status.success() {
			bail!("`git` command failed: {status}");
		}

		Ok(())
	}
}
