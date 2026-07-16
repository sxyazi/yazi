use anyhow::{Result, bail};
use yazi_config::{YAZI, popup::ConfirmCfg};
use yazi_dds::Pubsub;
use yazi_fs::{FilesOp, file::File};
use yazi_macro::{act, err, input, ok_or_not_found, succ};
use yazi_parser::mgr::RenameForm;
use yazi_proxy::{ConfirmProxy, MgrProxy};
use yazi_shared::{data::Data, id::Id, url::{UrlBuf, UrlLike}};
use yazi_vfs::{VfsFile, engine};
use yazi_watcher::WATCHER;
use yazi_widgets::input::InputEvent;

use crate::{Actor, Ctx};

pub struct Rename;

impl Actor for Rename {
	type Form = RenameForm;

	const NAME: &str = "rename";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		if !form.hovered && !cx.tab().selected.is_empty() {
			return act!(mgr:bulk_rename, cx);
		}

		let Some(hovered) = cx.hovered() else { succ!() };

		let name = Self::empty_url_part(&hovered.url, &form.empty);
		let cursor = match form.cursor.as_ref() {
			"start" => Some(0),
			"before_ext" => name
				.chars()
				.rev()
				.position(|c| c == '.')
				.filter(|_| hovered.is_file())
				.map(|i| name.chars().count() - i - 1)
				.filter(|&i| i != 0),
			_ => None,
		};

		let (tab, old) = (cx.tab().id, hovered.url_owned());
		let mut input =
			input!(cx, YAZI.input.rename(hovered.is_dir()).with_value(name).with_cursor(cursor))?;

		tokio::spawn(async move {
			let Some(InputEvent::Submit(name)) = input.recv().await else { return Ok(()) };
			if name.is_empty() {
				return Ok(());
			}

			let Some(Ok(new)) = old.parent().map(|u| u.try_join(name)) else {
				bail!("Failed to join new name with parent directory");
			};

			if form.force || Self::try_ask(&old, &new).await? {
				Self::r#do(tab, old, new).await?;
			}
			Ok::<(), anyhow::Error>(())
		});
		succ!();
	}
}

impl Rename {
	async fn r#do(tab: Id, old: UrlBuf, new: UrlBuf) -> Result<()> {
		let Some((old_p, old_k)) = old.pair2() else { return Ok(()) };
		let Some(_) = new.pair2() else { return Ok(()) };
		let _permit = WATCHER.acquire().await.unwrap();

		let overwritten = engine::casefold(&new).await;
		engine::rename(&old, &new).await?;

		if let Ok(u) = overwritten
			&& u != new
			&& let Some((parent, key)) = u.pair2()
		{
			ok_or_not_found!(engine::rename(&u, &new).await);
			FilesOp::Deleting(parent.to_owned(), [key.into()].into()).emit();
		}

		let new = engine::casefold(&new).await?;
		let Some((new_p, new_k)) = new.pair2() else { return Ok(()) };

		let file = File::new(&new).await?;
		if new_p == old_p {
			FilesOp::Upserting(old_p.into(), [(old_k.into(), file)].into()).emit();
		} else {
			FilesOp::Deleting(old_p.into(), [old_k.into()].into()).emit();
			FilesOp::Upserting(new_p.into(), [(new_k.into(), file)].into()).emit();
		}

		MgrProxy::reveal(&new);
		err!(Pubsub::pub_after_rename(tab, &old, &new));
		Ok(())
	}

	async fn try_ask(old: &UrlBuf, new: &UrlBuf) -> Result<bool> {
		let Some(file) = File::maybe_new(&new).await? else {
			return Ok(true);
		};

		Ok(
			engine::must_identical(old, new).await
				|| ConfirmProxy::show(ConfirmCfg::overwrite(&file)).await,
		)
	}

	fn empty_url_part(url: &UrlBuf, by: &str) -> String {
		if by == "all" {
			return String::new();
		}

		let ext = url.ext();
		match by {
			"stem" => ext.map_or_else(String::new, |s| format!(".{}", s.to_string_lossy())),
			"ext" if ext.is_some() => format!("{}.", url.stem().unwrap().to_string_lossy()),
			"dot_ext" if ext.is_some() => url.stem().unwrap().to_string_lossy().into_owned(),
			_ => url.name().unwrap_or_default().to_string_lossy().into_owned(),
		}
	}
}
