use std::collections::HashMap;

use anyhow::Result;
use tokio::fs;
use yazi_config::popup::InputCfg;
use yazi_proxy::{InputProxy, ManagerProxy, WATCHER};
use yazi_shared::{event::Cmd, fs::{accessible, File, FilesOp, Url}};

use crate::manager::Manager;

pub struct Opt {
	force:  bool,
	empty:  String,
	cursor: String,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			force:  c.named.contains_key("force"),
			empty:  c.take_name("empty").unwrap_or_default(),
			cursor: c.take_name("cursor").unwrap_or_default(),
		}
	}
}

impl Manager {
	pub fn rename(&mut self, opt: impl Into<Opt>) {
		if !self.active_mut().try_escape_visual() {
			return;
		} else if !self.active().selected.is_empty() {
			return self.bulk_rename();
		}

		let Some(hovered) = self.hovered().map(|h| h.url()) else {
			return;
		};

		let opt = opt.into() as Opt;
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

		tokio::spawn(async move {
			let mut result = InputProxy::show(InputCfg::rename().with_value(name).with_cursor(cursor));
			let Some(Ok(name)) = result.recv().await else {
				return;
			};

			let new = hovered.parent().unwrap().join(name);
			if opt.force || !accessible(&new).await {
				Self::rename_do(hovered, Url::from(new)).await.ok();
				return;
			}

			let mut result = InputProxy::show(InputCfg::overwrite());
			if let Some(Ok(choice)) = result.recv().await {
				if choice == "y" || choice == "Y" {
					Self::rename_do(hovered, Url::from(new)).await.ok();
				}
			};
		});
	}

	async fn rename_do(old: Url, new: Url) -> Result<()> {
		let _permit = WATCHER.acquire().await.unwrap();

		fs::rename(&old, &new).await?;
		if old.parent() != new.parent() {
			return Ok(());
		}

		let file = File::from(new.clone()).await?;
		FilesOp::Deleting(file.parent().unwrap(), vec![new.clone()]).emit();
		FilesOp::Upserting(file.parent().unwrap(), HashMap::from_iter([(old, file)])).emit();
		Ok(ManagerProxy::hover(Some(new)))
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
