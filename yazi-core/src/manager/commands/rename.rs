use std::{collections::BTreeMap, ffi::OsStr, io::{stdout, BufWriter, Write}, path::PathBuf};

use anyhow::{anyhow, bail, Result};
use tokio::{fs::{self, OpenOptions}, io::{stdin, AsyncReadExt, AsyncWriteExt}};
use yazi_config::{popup::InputCfg, OPEN, PREVIEW};
use yazi_plugin::external::{self, ShellOpt};
use yazi_scheduler::{Scheduler, BLOCKER};
use yazi_shared::{event::Cmd, fs::{max_common_root, File, FilesOp, Url}, term::Term, Defer};

use crate::{input::Input, manager::Manager};

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

	async fn rename_and_hover(old: Url, new: Url) -> Result<()> {
		fs::rename(&old, &new).await?;
		if old.parent() != new.parent() {
			return Ok(());
		}

		let file = File::from(new.clone()).await?;
		FilesOp::Deleting(file.parent().unwrap(), vec![new.clone()]).emit();
		FilesOp::Upserting(file.parent().unwrap(), BTreeMap::from_iter([(old, file)])).emit();
		Ok(Self::_hover(Some(new)))
	}

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
			let mut result = Input::_show(InputCfg::rename().with_value(name).with_cursor(cursor));
			let Some(Ok(name)) = result.recv().await else {
				return;
			};

			let new = hovered.parent().unwrap().join(name);
			if opt.force || fs::symlink_metadata(&new).await.is_err() {
				Self::rename_and_hover(hovered, Url::from(new)).await.ok();
				return;
			}

			let mut result = Input::_show(InputCfg::overwrite());
			if let Some(Ok(choice)) = result.recv().await {
				if choice == "y" || choice == "Y" {
					Self::rename_and_hover(hovered, Url::from(new)).await.ok();
				}
			};
		});
	}

	fn bulk_rename(&self) {
		let old: Vec<_> = self.selected_or_hovered();

		let root = max_common_root(&old);
		let old: Vec<_> = old.into_iter().map(|p| p.strip_prefix(&root).unwrap().to_owned()).collect();

		let tmp = PREVIEW.tmpfile("bulk");
		tokio::spawn(async move {
			let Some(opener) = OPEN.block_opener("bulk.txt", "text/plain") else {
				bail!("No opener for bulk rename");
			};

			{
				let s = old.iter().map(|o| o.as_os_str()).collect::<Vec<_>>().join(OsStr::new("\n"));
				OpenOptions::new()
					.write(true)
					.create_new(true)
					.open(&tmp)
					.await?
					.write_all(s.as_encoded_bytes())
					.await?;
			}

			let _guard = BLOCKER.acquire().await.unwrap();
			let _defer = Defer::new(|| {
				Scheduler::app_resume();
				tokio::spawn(fs::remove_file(tmp.clone()))
			});
			Scheduler::app_stop().await;

			let mut child = external::shell(ShellOpt {
				cmd:    (*opener.exec).into(),
				args:   vec![tmp.to_owned().into()],
				piped:  false,
				orphan: false,
			})?;
			child.wait().await?;

			let new: Vec<_> = fs::read_to_string(&tmp).await?.lines().map(PathBuf::from).collect();
			Self::bulk_rename_do(root, old, new).await
		});
	}

	async fn bulk_rename_do(root: PathBuf, old: Vec<PathBuf>, new: Vec<PathBuf>) -> Result<()> {
		Term::clear(&mut stdout())?;
		if old.len() != new.len() {
			println!("Number of old and new differ, press ENTER to exit");
			stdin().read_exact(&mut [0]).await?;
			return Ok(());
		}

		let todo: Vec<_> = old.into_iter().zip(new).filter(|(o, n)| o != n).collect();
		if todo.is_empty() {
			return Ok(());
		}

		{
			let mut stdout = BufWriter::new(stdout().lock());
			for (o, n) in &todo {
				writeln!(stdout, "{} -> {}", o.display(), n.display())?;
			}
			write!(stdout, "Continue to rename? (y/N): ")?;
			stdout.flush()?;
		}

		let mut buf = [0; 10];
		_ = stdin().read(&mut buf).await?;
		if buf[0] != b'y' && buf[0] != b'Y' {
			return Ok(());
		}

		let mut failed = vec![];
		for (o, n) in todo {
			if fs::symlink_metadata(&n).await.is_ok() {
				failed.push((o, n, anyhow!("Destination already exists")));
				continue;
			}
			if let Err(e) = fs::rename(root.join(&o), root.join(&n)).await {
				failed.push((o, n, e.into()));
			}
		}
		if failed.is_empty() {
			return Ok(());
		}

		Term::clear(&mut stdout())?;
		{
			let mut stdout = BufWriter::new(stdout().lock());
			writeln!(stdout, "Failed to rename:")?;
			for (o, n, e) in failed {
				writeln!(stdout, "{} -> {}: {e}", o.display(), n.display())?;
			}
			writeln!(stdout, "\nPress ENTER to exit")?;
			stdout.flush()?;
		}

		stdin().read_exact(&mut [0]).await?;
		Ok(())
	}
}
