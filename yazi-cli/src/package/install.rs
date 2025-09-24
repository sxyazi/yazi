use anyhow::Result;

use super::{Dependency, Git};
use crate::shared::must_exists;

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
		if self.rev.is_empty() {
			self.rev = Git::revision(&path).await?;
		}

		Ok(())
	}
}
