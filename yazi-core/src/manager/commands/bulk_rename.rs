use std::{borrow::Cow, collections::HashMap, ffi::{OsStr, OsString}, io::{stderr, BufWriter, Write}, path::PathBuf};

use anyhow::{anyhow, Result};
use tokio::{fs::{self, OpenOptions}, io::{stdin, AsyncReadExt, AsyncWriteExt}};
use yazi_config::{OPEN, PREVIEW};
use yazi_proxy::{AppProxy, TasksProxy, HIDER, WATCHER};
use yazi_shared::{fs::{accessible, max_common_root, File, FilesOp, Url}, term::Term, Defer};

use crate::manager::Manager;

impl Manager {
	pub(super) fn bulk_rename(&self) {
		let Some(opener) = OPEN.block_opener("bulk.txt", "text/plain") else {
			return AppProxy::notify_warn("Bulk rename", "No text opener found");
		};

		let cwd = self.cwd().clone();
		let old: Vec<_> = self.selected_or_hovered(true);

		let root = max_common_root(&old);
		let old: Vec<_> = old.into_iter().map(|p| p.strip_prefix(&root).unwrap().to_owned()).collect();

		tokio::spawn(async move {
			let tmp = PREVIEW.tmpfile("bulk");
			let s = old.iter().map(|o| o.as_os_str()).collect::<Vec<_>>().join(OsStr::new("\n"));
			OpenOptions::new()
				.write(true)
				.create_new(true)
				.open(&tmp)
				.await?
				.write_all(s.as_encoded_bytes())
				.await?;

			let _defer1 = Defer::new(|| tokio::spawn(fs::remove_file(tmp.clone())));
			TasksProxy::process_exec(vec![OsString::new(), tmp.to_owned().into()], Cow::Borrowed(opener))
				.await;

			let _permit = HIDER.acquire().await.unwrap();
			let _defer2 = Defer::new(AppProxy::resume);
			AppProxy::stop().await;

			let new: Vec<_> = fs::read_to_string(&tmp).await?.lines().map(PathBuf::from).collect();
			Self::bulk_rename_do(cwd, root, old, new).await
		});
	}

	async fn bulk_rename_do(
		cwd: Url,
		root: PathBuf,
		old: Vec<PathBuf>,
		new: Vec<PathBuf>,
	) -> Result<()> {
		Term::clear(&mut stderr())?;
		if old.len() != new.len() {
			eprintln!("Number of old and new differ, press ENTER to exit");
			stdin().read_exact(&mut [0]).await?;
			return Ok(());
		}

		let todo: Vec<_> = old.into_iter().zip(new).filter(|(o, n)| o != n).collect();
		if todo.is_empty() {
			return Ok(());
		}

		{
			let mut stderr = BufWriter::new(stderr().lock());
			for (o, n) in &todo {
				writeln!(stderr, "{} -> {}", o.display(), n.display())?;
			}
			write!(stderr, "Continue to rename? (y/N): ")?;
			stderr.flush()?;
		}

		let mut buf = [0; 10];
		_ = stdin().read(&mut buf).await?;
		if buf[0] != b'y' && buf[0] != b'Y' {
			return Ok(());
		}

		let permit = WATCHER.acquire().await.unwrap();
		let (mut failed, mut succeeded) = (Vec::new(), HashMap::with_capacity(todo.len()));
		for (o, n) in todo {
			let (old, new) = (root.join(&o), root.join(&n));

			if accessible(&new).await {
				failed.push((o, n, anyhow!("Destination already exists")));
			} else if let Err(e) = fs::rename(&old, &new).await {
				failed.push((o, n, e.into()));
			} else if let Ok(f) = File::from(new.into()).await {
				succeeded.insert(Url::from(old), f);
			} else {
				failed.push((o, n, anyhow!("Failed to retrieve file info")));
			}
		}

		if !succeeded.is_empty() {
			FilesOp::Upserting(cwd, succeeded).emit();
		}
		drop(permit);

		if !failed.is_empty() {
			Self::output_failed(failed).await?;
		}
		Ok(())
	}

	async fn output_failed(failed: Vec<(PathBuf, PathBuf, anyhow::Error)>) -> Result<()> {
		Term::clear(&mut stderr())?;

		{
			let mut stderr = BufWriter::new(stderr().lock());
			writeln!(stderr, "Failed to rename:")?;
			for (o, n, e) in failed {
				writeln!(stderr, "{} -> {}: {e}", o.display(), n.display())?;
			}
			writeln!(stderr, "\nPress ENTER to exit")?;
			stderr.flush()?;
		}

		stdin().read_exact(&mut [0]).await?;
		Ok(())
	}
}
