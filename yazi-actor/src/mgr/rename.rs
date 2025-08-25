use anyhow::Result;
use yazi_config::popup::{ConfirmCfg, InputCfg};
use yazi_dds::Pubsub;
use yazi_fs::{File, FilesOp, maybe_exists, ok_or_not_found, provider, realname};
use yazi_macro::{act, err, succ};
use yazi_parser::mgr::RenameOpt;
use yazi_proxy::{ConfirmProxy, InputProxy, MgrProxy};
use yazi_shared::{Id, event::Data, url::{UrlBuf, UrnBuf}};
use yazi_watcher::WATCHER;

use crate::{Actor, Ctx};

pub struct Rename;

impl Actor for Rename {
	type Options = RenameOpt;

	const NAME: &str = "rename";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		if !opt.hovered && !cx.tab().selected.is_empty() {
			return act!(mgr:bulk_rename, cx);
		}

		let Some(hovered) = cx.hovered() else { succ!() };

		let name = Self::empty_url_part(&hovered.url, &opt.empty);
		let cursor = match opt.cursor.as_ref() {
			"start" => Some(0),
			"before_ext" => name
				.chars()
				.rev()
				.position(|c| c == '.')
				.filter(|_| !hovered.is_dir())
				.map(|i| name.chars().count() - i - 1)
				.filter(|&i| i != 0),
			_ => None,
		};

		let (tab, old) = (cx.tab().id, hovered.url_owned());
		let mut input = InputProxy::show(InputCfg::rename().with_value(name).with_cursor(cursor));

		tokio::spawn(async move {
			let Some(Ok(name)) = input.recv().await else { return };
			if name.is_empty() {
				return;
			}

			let new = old.parent_url().unwrap().join(name);
			if opt.force || !maybe_exists(&new).await || provider::must_identical(&old, &new).await {
				Self::r#do(tab, old, new).await.ok();
			} else if ConfirmProxy::show(ConfirmCfg::overwrite(&new)).await {
				Self::r#do(tab, old, new).await.ok();
			}
		});
		succ!();
	}
}

impl Rename {
	async fn r#do(tab: Id, old: UrlBuf, new: UrlBuf) -> Result<()> {
		let Some((p_old, n_old)) = old.pair() else { return Ok(()) };
		let Some((p_new, n_new)) = new.pair() else { return Ok(()) };
		let _permit = WATCHER.acquire().await.unwrap();

		let overwritten = realname(&new).await;
		provider::rename(&old, &new).await?;

		if let Some(o) = overwritten {
			ok_or_not_found(provider::rename(&p_new.join(&o), &new).await)?;
			FilesOp::Deleting(p_new.to_owned(), [UrnBuf::from(o)].into()).emit();
		}

		let file = File::new(&new).await?;
		if p_new == p_old {
			FilesOp::Upserting(p_old.into(), [(n_old, file)].into()).emit();
		} else {
			FilesOp::Deleting(p_old.into(), [n_old].into()).emit();
			FilesOp::Upserting(p_new.into(), [(n_new, file)].into()).emit();
		}

		MgrProxy::reveal(&new);
		err!(Pubsub::pub_after_rename(tab, &old, &new));
		Ok(())
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
