use anyhow::{Result, bail};
use yazi_config::{YAZI, popup::ConfirmCfg};
use yazi_fs::{FilesOp, file::File};
use yazi_macro::{input, ok_or_not_found, succ};
use yazi_parser::mgr::CreateForm;
use yazi_proxy::{ConfirmProxy, MgrProxy};
use yazi_shared::{data::Data, url::{UrlBuf, UrlLike}};
use yazi_vfs::{VfsFile, maybe_exists, provider};
use yazi_watcher::WATCHER;
use yazi_widgets::input::InputEvent;

use crate::{Actor, Ctx};

pub struct Create;

impl Actor for Create {
	type Form = CreateForm;

	const NAME: &str = "create";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let cwd = cx.cwd().to_owned();

		let target = if let Some(target) = form.target {
			let target_str = target.loc().to_str()?.to_string();
			if target_str.is_empty() { None } else { Some(target_str) }
		} else { None };

		let input = if target.is_some() {
			None
		} else {
			Some(input!(cx, YAZI.input.create(form.dir))?)
		};

		tokio::spawn(async move {

			let target = match input {
				Some(mut input) => {
					let Some(InputEvent::Submit(name)) = input.recv().await else { return };
					name
				},
				None => target.unwrap(),
			};

			if target.is_empty() {
				return;
			}

			let Ok(new) = cwd.try_join(&target) else {
				return;
			};

			if !form.force
				&& maybe_exists(&new).await
				&& !ConfirmProxy::show(ConfirmCfg::overwrite(&new)).await
			{
				return;
			}

			_ = Self::r#do(new, form.dir || target.ends_with('/') || target.ends_with('\\')).await;
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
