use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use tokio::fs;
use yazi_fs::{Xdg, copy_and_seal, maybe_exists, remove_dir_clean};
use yazi_macro::outln;

use super::Dependency;

impl Dependency {
	pub(super) async fn deploy(&mut self) -> Result<()> {
		let from = self.local().join(&self.child);

		self.header("Deploying package `{name}`")?;
		self.is_flavor = maybe_exists(&from.join("flavor.toml")).await;
		let to = if self.is_flavor {
			Xdg::config_dir().join(format!("flavors/{}", self.name))
		} else {
			Xdg::config_dir().join(format!("plugins/{}", self.name))
		};

		if maybe_exists(&to).await && self.hash != self.hash().await? {
			bail!(
				"The user has modified the contents of the `{}` package. For safety, the operation has been aborted.
Please manually delete it from your plugins/flavors directory and re-run the command.",
				self.name
			);
		}

		fs::create_dir_all(&to).await?;
		let files = if self.is_flavor {
			&["flavor.toml", "tmtheme.xml", "README.md", "preview.png", "LICENSE", "LICENSE-tmtheme"][..]
		} else {
			&["main.lua", "README.md", "LICENSE"][..]
		};

		for file in files {
			// TODO: remove this
			let (from, to) = if *file == "main.lua" {
				if maybe_exists(from.join(file)).await {
					(from.join(file), to.join(file))
				} else {
					(from.join("init.lua"), to.join("main.lua"))
				}
			} else {
				(from.join(file), to.join(file))
			};

			copy_and_seal(&from, &to)
				.await
				.with_context(|| format!("failed to copy `{}` to `{}`", from.display(), to.display()))?;
		}

		Self::deploy_assets(from.join("assets"), to.join("assets")).await?;

		outln!("Done!")?;
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
					copy_and_seal(&src, &dist).await.with_context(|| {
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
