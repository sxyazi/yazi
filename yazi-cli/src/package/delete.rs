use anyhow::{Result, bail};
use tokio::fs;
use yazi_fs::maybe_exists;
use yazi_macro::outln;

use super::Dependency;

impl Dependency {
	pub(super) async fn delete(&self) -> Result<()> {
		self.header("Deleting package `{name}`")?;

		let dir = self.target();
		if !maybe_exists(&dir).await {
			return Ok(outln!("Not found, skipping")?);
		}

		if self.hash != self.hash().await? {
			bail!(
				"You have modified the contents of the `{}` {}. For safety, the operation has been aborted.
Please manually delete it from: {}",
				self.name,
				if self.is_flavor { "flavor" } else { "plugin" },
				dir.display()
			);
		}

		fs::remove_dir_all(&dir).await?;
		Ok(())
	}
}
