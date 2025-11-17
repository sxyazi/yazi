use anyhow::{Context, Result};
use yazi_fs::{ok_or_not_found, provider::{Provider, local::Local}};
use yazi_macro::outln;

use super::Dependency;
use crate::shared::{maybe_exists, remove_sealed};

impl Dependency {
	pub(super) async fn delete(&self) -> Result<()> {
		self.header("Deleting package `{name}`")?;

		let dir = self.target();
		if !maybe_exists(&dir).await {
			return Ok(outln!("Not found, skipping")?);
		}

		self.hash_check().await?;
		self.delete_assets().await?;
		self.delete_sources().await?;

		Ok(())
	}

	pub(super) async fn delete_assets(&self) -> Result<()> {
		let assets = self.target().join("assets");
		match tokio::fs::read_dir(&assets).await {
			Ok(mut it) => {
				while let Some(entry) = it.next_entry().await? {
					remove_sealed(&entry.path())
						.await
						.with_context(|| format!("failed to remove `{}`", entry.path().display()))?;
				}
			}
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
			Err(e) => Err(e).context(format!("failed to read `{}`", assets.display()))?,
		};

		Local::regular(&assets).remove_dir_clean().await;
		Ok(())
	}

	pub(super) async fn delete_sources(&self) -> Result<()> {
		let dir = self.target();
		let files =
			if self.is_flavor { Self::flavor_files() } else { Self::plugin_files(&dir).await? };

		for path in files.iter().map(|s| dir.join(s)) {
			ok_or_not_found(remove_sealed(&path).await)
				.with_context(|| format!("failed to delete `{}`", path.display()))?;
		}

		if ok_or_not_found(Local::regular(&dir).remove_dir().await).is_ok() {
			outln!("Done!")?;
		} else {
			outln!(
				"Done!
For safety, user data has been preserved, please manually delete them within: {}",
				dir.display()
			)?;
		}
		Ok(())
	}
}
