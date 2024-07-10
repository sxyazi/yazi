use std::collections::HashMap;

use anyhow::Result;
use tokio::fs;
use yazi_config::popup::InputCfg;
use yazi_proxy::{InputProxy, TabProxy, WATCHER};
use yazi_shared::{event::Cmd, fs::{maybe_exists, ok_or_not_found, symlink_realpath, File, FilesOp, Url}};

use crate::manager::Manager;

pub struct Opt {
	force: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { force: c.bool("force") } }
}

impl Manager {
	pub fn create(&self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		let cwd = self.cwd().to_owned();
		tokio::spawn(async move {
			let mut result = InputProxy::show(InputCfg::create());
			let Some(Ok(name)) = result.recv().await else {
				return Ok(());
			};
			if name.is_empty() {
				return Ok(());
			}

			let new = cwd.join(&name);
			if !opt.force && maybe_exists(&new).await {
				match InputProxy::show(InputCfg::overwrite()).recv().await {
					Some(Ok(c)) if c == "y" || c == "Y" => (),
					_ => return Ok(()),
				}
			}

			Self::create_do(new, name.ends_with('/') || name.ends_with('\\')).await
		});
	}

	async fn create_do(new: Url, dir: bool) -> Result<()> {
		let Some(parent) = new.parent_url() else { return Ok(()) };
		let _permit = WATCHER.acquire().await.unwrap();

		if dir {
			fs::create_dir_all(&new).await?;
		} else if let Ok(real) = symlink_realpath(&new).await {
			ok_or_not_found(fs::remove_file(&new).await)?;
			FilesOp::Deleting(parent.clone(), vec![Url::from(real)]).emit();
			fs::File::create(&new).await?;
		} else {
			fs::create_dir_all(&parent).await.ok();
			ok_or_not_found(fs::remove_file(&new).await)?;
			fs::File::create(&new).await?;
		}

		if let Ok(f) = File::from(new.clone()).await {
			FilesOp::Upserting(parent, HashMap::from_iter([(f.url(), f)])).emit();
			TabProxy::reveal(&new)
		}
		Ok(())
	}
}
