use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use tokio::fs;
use yazi_shared::{Xdg, fs::{maybe_exists, must_exists, remove_dir_clean}};

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
				.with_context(|| format!("failed to copy `{}` to `{}`", from.display(), to.display()))?;
		}

		Self::deploy_assets(from.join("assets"), to.join("assets")).await?;

		println!("Done!");
		Ok(())
	}

	async fn deploy_assets(from: PathBuf, to: PathBuf) -> Result<()> {
		use std::io::ErrorKind::NotFound;

		match fs::read_dir(&to).await {
			Ok(mut it) => {
				while let Some(entry) = it.next_entry().await? {
					fs::remove_file(entry.path())
						.await
						.with_context(|| format!("failed to remove `{}`", entry.path().display()))?;
				}
			}
			Err(e) if e.kind() == NotFound => {}
			Err(e) => Err(e).context(format!("failed to read `{}`", to.display()))?,
		};

		remove_dir_clean(&to).await;
		match fs::read_dir(&from).await {
			Ok(mut it) => {
				fs::create_dir_all(&to).await?;
				while let Some(entry) = it.next_entry().await? {
					let (src, dist) = (entry.path(), to.join(entry.file_name()));
					fs::copy(&src, &dist).await.with_context(|| {
						format!("failed to copy `{}` to `{}`", src.display(), dist.display())
					})?;
				}
			}
			Err(e) if e.kind() == NotFound => {}
			Err(e) => Err(e).context(format!("failed to read `{}`", from.display()))?,
		}

		Ok(())
	}
}
