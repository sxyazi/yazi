use anyhow::Result;
use yazi_config::popup::{ConfirmCfg, InputCfg};
use yazi_fs::{File, FilesOp, maybe_exists, ok_or_not_found, provider, realname};
use yazi_macro::succ;
use yazi_parser::mgr::CreateOpt;
use yazi_proxy::{ConfirmProxy, InputProxy, MgrProxy};
use yazi_shared::{event::Data, url::{UrlBuf, UrnBuf}};
use yazi_watcher::WATCHER;

use crate::{Actor, Ctx};

pub struct Create;

impl Actor for Create {
	type Options = CreateOpt;

	const NAME: &str = "create";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let cwd = cx.cwd().to_owned();
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

			_ = Self::r#do(new, opt.dir || name.ends_with('/') || name.ends_with('\\')).await;
		});
		succ!();
	}
}

impl Create {
	async fn r#do(new: UrlBuf, dir: bool) -> Result<()> {
		let Some(parent) = new.parent_url() else { return Ok(()) };
		let _permit = WATCHER.acquire().await.unwrap();

		if dir {
			provider::create_dir_all(&new).await?;
		} else if let Some(real) = realname(&new).await {
			ok_or_not_found(provider::remove_file(&new).await)?;
			FilesOp::Deleting(parent.to_owned(), [UrnBuf::from(real)].into()).emit();
			provider::create(&new).await?;
		} else {
			provider::create_dir_all(&parent).await.ok();
			ok_or_not_found(provider::remove_file(&new).await)?;
			provider::create(&new).await?;
		}

		if let Ok(f) = File::new(&new).await {
			FilesOp::Upserting(parent.into(), [(f.urn().to_owned(), f)].into()).emit();
			MgrProxy::reveal(&new)
		}
		Ok(())
	}
}
