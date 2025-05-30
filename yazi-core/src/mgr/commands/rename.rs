use std::borrow::Cow;

use anyhow::Result;
use tokio::fs;
use yazi_config::popup::{ConfirmCfg, InputCfg};
use yazi_dds::Pubsub;
use yazi_fs::{File, FilesOp, maybe_exists, ok_or_not_found, paths_to_same_file, realname};
use yazi_proxy::{ConfirmProxy, InputProxy, TabProxy, WATCHER};
use yazi_shared::{Id, event::CmdCow, url::{Url, UrnBuf}};

use crate::mgr::Mgr;

struct Opt {
	hovered: bool,
	force:   bool,
	empty:   Cow<'static, str>,
	cursor:  Cow<'static, str>,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			hovered: c.bool("hovered"),
			force:   c.bool("force"),
			empty:   c.take_str("empty").unwrap_or_default(),
			cursor:  c.take_str("cursor").unwrap_or_default(),
		}
	}
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn rename(&mut self, opt: Opt) {
		if !self.active_mut().try_escape_visual() {
			return;
		} else if !opt.hovered && !self.active().selected.is_empty() {
			return self.bulk_rename();
		}

		let Some(hovered) = self.hovered() else { return };
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

		let old = hovered.url_owned();
		let tab = self.tabs.active().id;
		let mut input = InputProxy::show(InputCfg::rename().with_value(name).with_cursor(cursor));

		tokio::spawn(async move {
			let Some(Ok(name)) = input.recv().await else { return };
			if name.is_empty() {
				return;
			}

			let new = Url::from(old.parent().unwrap().join(name));
			if opt.force || !maybe_exists(&new).await || paths_to_same_file(&old, &new).await {
				Self::rename_do(tab, old, new).await.ok();
			} else if ConfirmProxy::show(ConfirmCfg::overwrite(&new)).await {
				Self::rename_do(tab, old, new).await.ok();
			}
		});
	}

	async fn rename_do(tab: Id, old: Url, new: Url) -> Result<()> {
		let Some((p_old, n_old)) = old.pair() else { return Ok(()) };
		let Some((p_new, n_new)) = new.pair() else { return Ok(()) };
		let _permit = WATCHER.acquire().await.unwrap();

		let overwritten = realname(&new).await;
		fs::rename(&old, &new).await?;

		if let Some(o) = overwritten {
			ok_or_not_found(fs::rename(p_new.join(&o), &new).await)?;
			FilesOp::Deleting(p_new.clone(), [UrnBuf::from(o)].into()).emit();
		}

		let file = File::new(new.clone()).await?;
		if p_new == p_old {
			FilesOp::Upserting(p_old, [(n_old, file)].into()).emit();
		} else {
			FilesOp::Deleting(p_old, [n_old].into()).emit();
			FilesOp::Upserting(p_new, [(n_new, file)].into()).emit();
		}

		TabProxy::reveal(&new);
		Pubsub::pub_from_rename(tab, &old, &new);
		Ok(())
	}

	fn empty_url_part(url: &Url, by: &str) -> String {
		if by == "all" {
			return String::new();
		}

		let ext = url.extension();
		match by {
			"stem" => ext.map_or_else(String::new, |s| format!(".{}", s.to_string_lossy())),
			"ext" if ext.is_some() => format!("{}.", url.file_stem().unwrap().to_string_lossy()),
			"dot_ext" if ext.is_some() => url.file_stem().unwrap().to_string_lossy().into_owned(),
			_ => url.name().to_string_lossy().into_owned(),
		}
	}
}
