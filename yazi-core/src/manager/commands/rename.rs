use std::collections::HashMap;

use anyhow::Result;
use tokio::fs;
use yazi_config::popup::{ConfirmCfg, InputCfg};
use yazi_dds::Pubsub;
use yazi_proxy::{ConfirmProxy, InputProxy, TabProxy, WATCHER};
use yazi_shared::{event::Cmd, fs::{maybe_exists, ok_or_not_found, paths_to_same_file, symlink_realpath, File, FilesOp, Url}};

use crate::manager::Manager;

pub struct Opt {
	hovered: bool,
	force:   bool,
	empty:   String,
	cursor:  String,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			hovered: c.bool("hovered"),
			force:   c.bool("force"),
			empty:   c.take_str("empty").unwrap_or_default(),
			cursor:  c.take_str("cursor").unwrap_or_default(),
		}
	}
}

impl Manager {
	pub fn rename(&mut self, opt: impl Into<Opt>) {
		if !self.active_mut().try_escape_visual() {
			return;
		}
		let Some(hovered) = self.hovered().map(|h| h.url_owned()) else {
			return;
		};

		let opt = opt.into() as Opt;
		if !opt.hovered && !self.active().selected.is_empty() {
			return self.bulk_rename();
		}

		let name = Self::empty_url_part(&hovered, &opt.empty);
		let cursor = match opt.cursor.as_str() {
			"start" => Some(0),
			"before_ext" => name
				.chars()
				.rev()
				.position(|c| c == '.')
				.map(|i| name.chars().count() - i - 1)
				.filter(|&i| i != 0),
			_ => None,
		};

		let tab = self.tabs.cursor;
		tokio::spawn(async move {
			let mut result = InputProxy::show(InputCfg::rename().with_value(name).with_cursor(cursor));
			let Some(Ok(name)) = result.recv().await else {
				return;
			};

			if name.is_empty() {
				return;
			}

			let new = Url::from(hovered.parent().unwrap().join(name));
			if opt.force || !maybe_exists(&new).await || paths_to_same_file(&hovered, &new).await {
				Self::rename_do(tab, hovered, new).await.ok();
			} else if ConfirmProxy::show(ConfirmCfg::overwrite(&new)).await {
				Self::rename_do(tab, hovered, new).await.ok();
			}
		});
	}

	async fn rename_do(tab: usize, old: Url, new: Url) -> Result<()> {
		let Some(p_old) = old.parent_url() else { return Ok(()) };
		let Some(p_new) = new.parent_url() else { return Ok(()) };
		let _permit = WATCHER.acquire().await.unwrap();

		let overwritten = symlink_realpath(&new).await;
		fs::rename(&old, &new).await?;

		if let Ok(o) = overwritten {
			ok_or_not_found(fs::rename(&o, &new).await)?;
			FilesOp::Deleting(p_new.clone(), vec![Url::from(o)]).emit();
		}
		Pubsub::pub_from_rename(tab, &old, &new);

		let file = File::from(new.clone()).await?;
		FilesOp::Deleting(p_old, vec![old]).emit();
		FilesOp::Upserting(p_new, HashMap::from_iter([(new.clone(), file)])).emit();
		Ok(TabProxy::reveal(&new))
	}

	fn empty_url_part(url: &Url, by: &str) -> String {
		if by == "all" {
			return String::new();
		}

		let ext = url.extension();
		match by {
			"stem" => ext.map_or_else(String::new, |s| format!(".{}", s.to_string_lossy().into_owned())),
			"ext" if ext.is_some() => format!("{}.", url.file_stem().unwrap().to_string_lossy()),
			"dot_ext" if ext.is_some() => url.file_stem().unwrap().to_string_lossy().into_owned(),
			_ => url.file_name().map_or_else(String::new, |s| s.to_string_lossy().into_owned()),
		}
	}
}
