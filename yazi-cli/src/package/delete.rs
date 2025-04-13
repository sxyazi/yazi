use anyhow::{Context, Result};
use tokio::fs;
use yazi_fs::{maybe_exists, ok_or_not_found, remove_dir_clean, remove_sealed};
use yazi_macro::outln;

use super::Dependency;

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
		match fs::read_dir(&assets).await {
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

		remove_dir_clean(&assets).await;
		Ok(())
	}

	pub(super) async fn delete_sources(&self) -> Result<()> {
		let dir = self.target();
		let files = if self.is_flavor {
			&["flavor.toml", "tmtheme.xml", "README.md", "preview.png", "LICENSE", "LICENSE-tmtheme"][..]
		} else {
			&["main.lua", "README.md", "LICENSE"][..]
		};

		for p in files.iter().map(|&f| dir.join(f)) {
			ok_or_not_found(remove_sealed(&p).await)
				.with_context(|| format!("failed to delete `{}`", p.display()))?;
		}

		if ok_or_not_found(fs::remove_dir(&dir).await).is_ok() {
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
