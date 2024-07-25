use anyhow::Result;
use yazi_shared::fs::must_exists;

use super::{Git, Package};

impl Package {
	pub(super) async fn add(&mut self) -> Result<()> {
		self.header("Upgrading package `{name}`")?;

		let path = self.local();
		if !must_exists(&path).await {
			Git::clone(&self.remote(), &path).await?;
		} else {
			Git::pull(&path).await?;
		};

		self.commit = Git::hash(&path).await?;
		self.deploy().await
	}
}
