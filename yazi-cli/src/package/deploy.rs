use anyhow::{bail, Context, Result};
use tokio::fs;
use yazi_shared::{fs::{maybe_exists, must_exists}, Xdg};

use super::Package;

const TRACKER: &str = "DO_NOT_MODIFY_ANYTHING_IN_THIS_DIRECTORY";

impl Package {
	pub(super) async fn deploy(&mut self) -> Result<()> {
		let Some(name) = self.name().map(ToOwned::to_owned) else { bail!("Invalid package url") };
		let from = self.local().join(&self.child);

		self.header("Deploying package `{name}`")?;
		self.is_flavor = maybe_exists(&from.join("flavor.toml")).await;
		let to = if self.is_flavor {
			Xdg::config_dir().join(format!("flavors/{name}"))
		} else {
			Xdg::config_dir().join(format!("plugins/{name}"))
		};

		let tracker = to.join(TRACKER);
		if maybe_exists(&to).await && !must_exists(&tracker).await {
			bail!(
				"A user package with the same name `{name}` already exists.
For safety, please manually delete it from your plugin/flavor directory and re-run the command."
			);
		}

		fs::create_dir_all(&to).await?;
		fs::write(tracker, []).await?;

		let files = if self.is_flavor {
			&["flavor.toml", "tmtheme.xml", "README.md", "preview.png", "LICENSE", "LICENSE-tmtheme"][..]
		} else {
			&["init.lua", "README.md", "LICENSE"][..]
		};

		for file in files {
			let (from, to) = (from.join(file), to.join(file));

			fs::copy(&from, &to)
				.await
				.with_context(|| format!("Failed to copy `{}` to `{}`", from.display(), to.display()))?;
		}

		println!("Done!");
		Ok(())
	}
}
