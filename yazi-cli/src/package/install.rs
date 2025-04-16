use anyhow::Result;
use yazi_fs::must_exists;

use super::{Dependency, Git};

impl Dependency {
	pub(super) async fn install(&mut self) -> Result<()> {
		self.header("Fetching package `{name}`")?;

		let path = self.local();
		if must_exists(&path).await {
			Git::fetch(&path).await?;
		} else {
			Git::clone(&self.remote(), &path).await?;
		};

		if !self.rev.is_empty() {
			Git::checkout(&path, self.rev.trim_start_matches('=')).await?;
		}

		self.deploy().await?;
		self.rev = Git::revision(&path).await?;
		Ok(())
	}
}
