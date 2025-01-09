use anyhow::{Result, bail};
use tokio::fs;
use yazi_fs::must_exists;
use yazi_macro::outln;

use super::Dependency;

impl Dependency {
	pub(super) async fn delete(&self) -> Result<()> {
		self.header("Deleting package `{name}`")?;

		let path = self.deployed_directory();

		if must_exists(&path).await {
			fs::remove_dir_all(&path).await?;
		} else {
			bail!(
				"The package.toml file states that `{}` exists, but the directory was not found. The entry will be removed from package.toml.",
				self.name
			);
		}

		outln!("Done!")?;
		Ok(())
	}
}
