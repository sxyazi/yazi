use anyhow::Result;
use yazi_fs::must_exists;

use super::{Dependency, Git};

impl Dependency {
	pub(super) async fn add(&mut self) -> Result<()> {
		self.header("Upgrading package `{name}`")?;

		let path = self.local();
		if !must_exists(&path).await {
			Git::clone(&self.remote(), &path).await?;
		} else {
			Git::pull(&path).await?;
		};

		self.rev = Git::hash(&path).await?;
		self.deploy().await
	}
}
