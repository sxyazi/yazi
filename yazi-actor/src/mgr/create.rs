use std::pin::Pin;

use anyhow::{Result, bail};
use futures::{Stream, StreamExt};
use tokio_stream::wrappers::UnboundedReceiverStream;
use yazi_config::{YAZI, popup::ConfirmCfg};
use yazi_fs::{FilesOp, file::File};
use yazi_macro::{input, ok_or_not_found, succ};
use yazi_parser::mgr::CreateForm;
use yazi_proxy::{ConfirmProxy, MgrProxy};
use yazi_shared::{AnyAsciiChar, BytePredictor, data::Data, strand::{StrandBuf, StrandLike}, url::{UrlBuf, UrlLike}};
use yazi_vfs::{VfsFile, provider};
use yazi_watcher::WATCHER;

use crate::{Actor, Ctx};

pub struct Create;

impl Actor for Create {
	type Form = CreateForm;

	const NAME: &str = "create";

	fn act(cx: &mut Ctx, CreateForm { target, dir, force }: Self::Form) -> Result<Data> {
		let cwd = cx.cwd().to_owned();

		let mut target: Pin<Box<dyn Stream<Item = StrandBuf> + Send>> = if target.is_empty() {
			let input = input!(cx, YAZI.input.create(dir))?;
			Box::pin(
				UnboundedReceiverStream::new(input).filter_map(|event| async { event.map(Into::into) }),
			)
		} else {
			Box::pin(tokio_stream::iter(vec![target]))
		};

		tokio::spawn(async move {
			let Some(name) = target.next().await else { return Ok(()) };
			if name.is_empty() {
				return Ok(());
			}

			let Ok(new) = cwd.try_join(&name) else {
				bail!("Failed to join new name with CWD");
			};

			if !force
				&& let Some(file) = File::maybe_new(&new).await?
				&& !ConfirmProxy::show(ConfirmCfg::overwrite(&file)).await
			{
				return Ok(());
			}

			let end_sep = AnyAsciiChar::SEP.predicate(*name.encoded_bytes().last().unwrap());
			Self::r#do(new, dir || end_sep).await
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
