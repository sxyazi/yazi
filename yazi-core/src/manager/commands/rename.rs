use std::{collections::BTreeMap, ffi::OsStr, io::{stdout, BufWriter, Write}, path::PathBuf};

use anyhow::{anyhow, bail, Result};
use tokio::{fs::{self, OpenOptions}, io::{stdin, AsyncReadExt, AsyncWriteExt}};
use yazi_config::{keymap::Exec, popup::InputOpt, OPEN, PREVIEW};
use yazi_shared::{max_common_root, Defer, Term, Url};

use crate::{emit, external::{self, ShellOpt}, files::{File, FilesOp}, manager::Manager, Event, BLOCKER};

pub struct Opt {
	force: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { force: e.named.contains_key("force") } }
}

impl Manager {
	async fn rename_and_hover(old: Url, new: Url) -> Result<()> {
		fs::rename(&old, &new).await?;
		if old.parent() != new.parent() {
			return Ok(());
		}

		let file = File::from(new.clone()).await?;
		emit!(Files(FilesOp::Replacing(file.parent().unwrap(), BTreeMap::from_iter([(old, file)]))));
		Ok(Self::_hover(Some(new)))
	}

	pub fn rename(&self, opt: impl Into<Opt>) -> bool {
		if self.active().in_selecting() {
			return self.bulk_rename();
		}

		let Some(hovered) = self.hovered().map(|h| h.url()) else {
			return false;
		};

		let opt = opt.into() as Opt;
		tokio::spawn(async move {
			let mut result =
				emit!(Input(InputOpt::rename().with_value(hovered.file_name().unwrap().to_string_lossy())));

			let Some(Ok(name)) = result.recv().await else {
				return;
			};

			let new = hovered.parent().unwrap().join(name);
			if opt.force || fs::symlink_metadata(&new).await.is_err() {
				Self::rename_and_hover(hovered, Url::from(new)).await.ok();
				return;
			}

			let mut result = emit!(Input(InputOpt::overwrite()));
			if let Some(Ok(choice)) = result.recv().await {
				if choice == "y" || choice == "Y" {
					Self::rename_and_hover(hovered, Url::from(new)).await.ok();
				}
			};
		});
		false
	}

	fn bulk_rename(&self) -> bool {
		let old: Vec<_> = self.selected().into_iter().map(|f| &f.url).collect();

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
				Event::Stop(false, None).emit();
				tokio::spawn(fs::remove_file(tmp.clone()))
			});
			emit!(Stop(true)).await;

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

		false
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
		stdin().read(&mut buf).await.ok();
		if buf[0] != b'y' && buf[0] != b'Y' {
			return Ok(());
		}

		let mut failed = Vec::new();
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
