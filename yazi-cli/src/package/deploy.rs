use anyhow::{Context, Result, bail};
use tokio::fs::{self, remove_dir_all};
use yazi_shared::{Xdg, fs::{copy_dir_all, maybe_exists, must_exists, remove_dir_clean}};

use super::{DiffAction, Package};
use crate::package::Git;

const TRACKER: &str = "DO_NOT_MODIFY_ANYTHING_IN_THIS_DIRECTORY";

impl Package {
	pub(super) async fn deploy(&mut self) -> Result<()> {
		let Some(name) = self.name().map(ToOwned::to_owned) else { bail!("Invalid package url") };
		let origin = self.local().join(&self.child);

		self.header("Deploying package `{name}`")?;
		self.is_flavor = maybe_exists(&origin.join("flavor.toml")).await;
		let dest = if self.is_flavor {
			Xdg::config_dir().join(format!("flavors/{name}"))
		} else {
			Xdg::config_dir().join(format!("plugins/{name}"))
		};

		let tracker = dest.join(TRACKER);
		if maybe_exists(&dest).await && !must_exists(&tracker).await {
			bail!(
				"A user package with the same name `{name}` already exists.
For safety, please manually delete it from your plugin/flavor directory and re-run the command."
			);
		}

		fs::create_dir_all(&dest).await?;

		let mut dirs = &["assets"][..];
		let mut diff_actions = vec![];
		if tracker.exists() {
			let prev_rev = String::from_utf8(fs::read(&tracker).await?)?;

			if prev_rev != self.rev {
				diff_actions = Git::diff(&self.local().join(&self.child), &prev_rev, dirs).await?;
			}
		}

		fs::write(tracker, &self.rev.trim_start_matches('=')).await?;

		let files = if self.is_flavor {
			&["flavor.toml", "tmtheme.xml", "README.md", "preview.png", "LICENSE", "LICENSE-tmtheme"][..]
		} else {
			&["init.lua", "README.md", "LICENSE"][..]
		};

		for file in files {
			let (from, to) = (origin.join(file), dest.join(file));

			fs::copy(&from, &to)
				.await
				.with_context(|| format!("failed to copy `{}` to `{}`", from.display(), to.display()))?;
		}

		for action in diff_actions.as_slice() {
			match action {
				DiffAction::Add { file } => {
					let (from, to) = (origin.join(file), dest.join(file));
					fs::create_dir_all(&to).await?;
					fs::copy(&from, &to).await.with_context(|| {
						format!("failed to copy `{}` to `{}`", from.display(), to.display())
					})?;
				}
				DiffAction::Delete { file } => {
					let file = dest.join(file);
					fs::remove_file(&file)
						.await
						.with_context(|| format!("failed to remove `{}`", file.display()))?;
				}
				DiffAction::Rename { old, new } => {
					let (from, to) = (dest.join(old), dest.join(new));
					fs::create_dir_all(&to).await?;
					fs::rename(&from, &to).await.with_context(|| {
						format!("failed to rename `{}` to `{}`", from.display(), to.display())
					})?;
				}
				DiffAction::Copy { old, new } => {
					let (from, to) = (dest.join(old), dest.join(new));
					fs::create_dir_all(&to).await?;
					fs::copy(&from, &to).await.with_context(|| {
						format!("failed to copy `{}` to `{}`", from.display(), to.display())
					})?;
				}
			};
		}

		remove_dir_clean(&dest).await;

		if !diff_actions.is_empty() {
			dirs = &[];
		}

		for dir in dirs {
			let (from, to) = (origin.join(dir), dest.join(dir));

			if !from.exists() {
				if to.exists() {
					remove_dir_all(&to)
						.await
						.with_context(|| format!("failed to prune no-longer exist folder {}", to.display()))?;
				}
				continue;
			}

			copy_dir_all(&from, &to).await.with_context(|| {
				format!("failed to copy dir `{}` to `{}`", from.display(), to.display())
			})?;
		}

		println!("Done!");
		Ok(())
	}
}
