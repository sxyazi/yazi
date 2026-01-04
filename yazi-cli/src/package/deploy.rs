use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use yazi_fs::provider::{Provider, local::Local};
use yazi_macro::outln;

use super::Dependency;
use crate::shared::{copy_and_seal, maybe_exists};

impl Dependency {
	pub(super) async fn deploy(&mut self) -> Result<()> {
		let from = self.local().join(&self.child);

		self.header("Deploying package `{name}`")?;
		self.is_flavor = maybe_exists(&from.join("flavor.toml")).await;

		let to = self.target();
		let exists = maybe_exists(&to).await;
		if exists {
			self.hash_check().await?;
		}

		Local::regular(&to).create_dir_all().await?;
		self.delete_assets().await?;

		let res1 = Self::deploy_assets(from.join("assets"), to.join("assets")).await;
		let res2 = Self::deploy_sources(&from, &to, self.is_flavor).await;
		if !exists && (res2.is_err() || res1.is_err()) {
			self.delete_assets().await?;
			self.delete_sources().await?;
		}

		Local::regular(&to).remove_dir_clean().await;
		self.hash = self.hash().await?;
		res2?;
		res1?;

		outln!("Done!")?;
		Ok(())
	}

	async fn deploy_assets(from: PathBuf, to: PathBuf) -> Result<()> {
		match tokio::fs::read_dir(&from).await {
			Ok(mut it) => {
				Local::regular(&to).create_dir_all().await?;
				while let Some(entry) = it.next_entry().await? {
					let (src, dist) = (entry.path(), to.join(entry.file_name()));
					copy_and_seal(&src, &dist).await.with_context(|| {
						format!("failed to copy `{}` to `{}`", src.display(), dist.display())
					})?;
				}
			}
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
			Err(e) => Err(e).context(format!("failed to read `{}`", from.display()))?,
		}
		Ok(())
	}

	async fn deploy_sources(from: &Path, to: &Path, is_flavor: bool) -> Result<()> {
		let files = if is_flavor { Self::flavor_files() } else { Self::plugin_files(from).await? };
		for file in files {
			let (from, to) = (from.join(&file), to.join(&file));
			copy_and_seal(&from, &to)
				.await
				.with_context(|| format!("failed to copy `{}` to `{}`", from.display(), to.display()))?;
		}
		Ok(())
	}
}
