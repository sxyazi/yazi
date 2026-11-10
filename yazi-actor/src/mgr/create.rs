use anyhow::{Result, bail};
use futures::future::Either;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::{YAZI, popup::ConfirmCfg};
use yazi_core::Ctx;
use yazi_dds::Pubsub;
use yazi_fs::{file::File, op::FilesOp};
use yazi_macro::{log_if_err, ok_or_not_found, succ};
use yazi_parser::mgr::CreateForm;
use yazi_proxy::{ConfirmProxy, MgrProxy};
use yazi_shared::{data::Data, id::Id, strand::StrandLike, url::{UrlBuf, UrlLike}};
use yazi_shim::bytes::{AnyAsciiChar, BytePredictor};
use yazi_vfs::{VfsFile, engine};
use yazi_watcher::WATCHER;

use crate::{Actor, input};

pub struct Create;

impl Actor for Create {
	type Form = CreateForm;

	const NAME: &str = "create";

	fn act(cx: &mut Ctx, CreateForm { target, dir, force }: Self::Form) -> Result<Data> {
		let (tab, cwd) = (cx.tab().id, cx.cwd().to_owned());

		let mut target = if target.is_empty() {
			let input = input!(cx, YAZI.input.create(dir))?;
			Either::Left(UnboundedReceiverStream::new(input).filter_map(|event| event.map(Into::into)))
		} else {
			Either::Right(tokio_stream::iter([target]))
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
			Self::r#do(tab, new, dir || end_sep).await
		});
		succ!();
	}
}

impl Create {
	async fn r#do(tab: Id, new: UrlBuf, dir: bool) -> Result<()> {
		let _permit = WATCHER.acquire().await.unwrap();

		if dir {
			engine::create_dir_all(&new).await?;
		} else if let Ok(real) = engine::casefold(&new).await
			&& let Some((trail, key)) = real.pair()
		{
			ok_or_not_found!(engine::remove_file(&new).await);
			FilesOp::Delete(trail.into(), [key.into()].into()).emit();
			engine::create_file(&new).await?;
		} else if let Some(parent) = new.parent() {
			engine::create_dir_all(parent).await.ok();
			ok_or_not_found!(engine::remove_file(&new).await);
			engine::create_file(&new).await?;
		} else {
			bail!("Cannot create file at root");
		}

		if let Ok(real) = engine::casefold(&new).await
			&& let Some((trail, key)) = real.pair()
		{
			let file = engine::file(&real).await?;
			FilesOp::Upsert(trail.into(), [(key.into(), file)].into()).emit();
			MgrProxy::reveal(&real);
			log_if_err!(Pubsub::pub_after_create(tab, &real));
		}

		Ok(())
	}
}
