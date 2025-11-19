use anyhow::{Result, bail};
use yazi_config::popup::{ConfirmCfg, InputCfg};
use yazi_fs::{File, FilesOp};
use yazi_macro::{ok_or_not_found, succ};
use yazi_parser::mgr::CreateOpt;
use yazi_proxy::{ConfirmProxy, InputProxy, MgrProxy};
use yazi_shared::{data::Data, url::{UrlBuf, UrlLike}};
use yazi_vfs::{VfsFile, maybe_exists, provider};
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

			let Ok(new) = cwd.try_join(&name) else {
				return;
			};

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
		let _permit = WATCHER.acquire().await.unwrap();

		if dir {
			provider::create_dir_all(&new).await?;
		} else if let Ok(real) = provider::casefold(&new).await
			&& let Some((parent, urn)) = real.pair()
		{
			ok_or_not_found!(provider::remove_file(&new).await);
			FilesOp::Deleting(parent.into(), [urn.into()].into()).emit();
			provider::create(&new).await?;
		} else if let Some(parent) = new.parent() {
			provider::create_dir_all(parent).await.ok();
			ok_or_not_found!(provider::remove_file(&new).await);
			provider::create(&new).await?;
		} else {
			bail!("Cannot create file at root");
		}

		if let Ok(real) = provider::casefold(&new).await
			&& let Some((parent, urn)) = real.pair()
		{
			let file = File::new(&real).await?;
			FilesOp::Upserting(parent.into(), [(urn.into(), file)].into()).emit();
			MgrProxy::reveal(&real);
		}

		Ok(())
	}
}
