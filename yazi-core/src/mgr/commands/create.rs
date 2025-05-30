use anyhow::Result;
use tokio::fs;
use yazi_config::popup::{ConfirmCfg, InputCfg};
use yazi_fs::{File, FilesOp, maybe_exists, ok_or_not_found, realname};
use yazi_proxy::{ConfirmProxy, InputProxy, TabProxy, WATCHER};
use yazi_shared::{event::CmdCow, url::{Url, UrnBuf}};

use crate::mgr::Mgr;

struct Opt {
	dir:   bool,
	force: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { dir: c.bool("dir"), force: c.bool("force") } }
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn create(&self, opt: Opt) {
		let cwd = self.cwd().to_owned();
		let mut input = InputProxy::show(InputCfg::create(opt.dir));

		tokio::spawn(async move {
			let Some(Ok(name)) = input.recv().await else { return };
			if name.is_empty() {
				return;
			}

			let new = cwd.join(&name);
			if !opt.force
				&& maybe_exists(&new).await
				&& !ConfirmProxy::show(ConfirmCfg::overwrite(&new)).await
			{
				return;
			}

			_ = Self::create_do(new, opt.dir || name.ends_with('/') || name.ends_with('\\')).await;
		});
	}

	async fn create_do(new: Url, dir: bool) -> Result<()> {
		let Some(parent) = new.parent_url() else { return Ok(()) };
		let _permit = WATCHER.acquire().await.unwrap();

		if dir {
			fs::create_dir_all(&new).await?;
		} else if let Some(real) = realname(&new).await {
			ok_or_not_found(fs::remove_file(&new).await)?;
			FilesOp::Deleting(parent.clone(), [UrnBuf::from(real)].into()).emit();
			fs::File::create(&new).await?;
		} else {
			fs::create_dir_all(&parent).await.ok();
			ok_or_not_found(fs::remove_file(&new).await)?;
			fs::File::create(&new).await?;
		}

		if let Ok(f) = File::new(new.clone()).await {
			FilesOp::Upserting(parent, [(f.urn_owned(), f)].into()).emit();
			TabProxy::reveal(&new)
		}
		Ok(())
	}
}
