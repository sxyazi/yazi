use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use tokio::fs;
use yazi_fs::{copy_and_seal, maybe_exists};
use yazi_macro::outln;

use super::Dependency;

impl Dependency {
	pub(super) async fn deploy(&mut self) -> Result<()> {
		let from = self.local().join(&self.child);

		self.header("Deploying package `{name}`")?;
		// TODO: maybe need to be removed
		let plugin_exists = maybe_exists(&from).await;
		if !plugin_exists {
			bail!("Plugin `{}` not found", self.name);
		}
		self.is_flavor = maybe_exists(&from.join("flavor.toml")).await;

		let to = self.target();
		if maybe_exists(&to).await && self.hash != self.hash().await? {
			bail!(
				"You have modified the contents of the `{}` {}. For safety, the operation has been aborted.
Please manually delete it from `{}` and re-run the command.",
				self.name,
				if self.is_flavor { "flavor" } else { "plugin" },
				to.display()
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

		self.delete_assets().await?;
		Self::deploy_assets(from.join("assets"), to.join("assets")).await?;

		self.hash = self.hash().await?;
		outln!("Done!")?;

		Ok(())
	}

	async fn deploy_assets(from: PathBuf, to: PathBuf) -> Result<()> {
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
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
			Err(e) => Err(e).context(format!("failed to read `{}`", from.display()))?,
		}
		Ok(())
	}
}
