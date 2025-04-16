use anyhow::Result;
use yazi_fs::must_exists;

use super::{Dependency, Git};

impl Dependency {
	pub(super) async fn add(&mut self) -> Result<()> {
		self.header("Upgrading package `{name}`")?;

		let path = self.local();
		if must_exists(&path).await {
			Git::pull(&path).await?;
		} else {
			Git::clone(&self.remote(), &path).await?;
		};

		self.deploy().await?;
		self.rev = Git::revision(&path).await?;
		Ok(())
	}
}
