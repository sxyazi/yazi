use std::{borrow::Cow, collections::HashMap, ffi::{OsStr, OsString}, io::{BufWriter, Write, stderr}, path::PathBuf};

use anyhow::{Result, anyhow};
use scopeguard::defer;
use tokio::{fs::{self, OpenOptions}, io::{AsyncReadExt, AsyncWriteExt, stdin}};
use yazi_config::{OPEN, PREVIEW};
use yazi_dds::Pubsub;
use yazi_fs::{File, FilesOp, max_common_root, maybe_exists, paths_to_same_file};
use yazi_proxy::{AppProxy, HIDER, TasksProxy, WATCHER};
use yazi_shared::{terminal_clear, url::Url};

use crate::mgr::Mgr;

impl Mgr {
	pub(super) fn bulk_rename(&self) {
		let Some(opener) = OPEN.block_opener("bulk-rename.txt", "text/plain") else {
			return AppProxy::notify_warn("Bulk rename", "No text opener found");
		};

		let old: Vec<_> = self.selected_or_hovered().collect();

		let root = max_common_root(&old);
		let old: Vec<_> = old.into_iter().map(|p| p.strip_prefix(&root).unwrap().to_owned()).collect();

		let cwd = self.cwd().clone();
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

			defer! { tokio::spawn(fs::remove_file(tmp.clone())); }
			TasksProxy::process_exec(Cow::Borrowed(opener), cwd, vec![
				OsString::new(),
				tmp.to_owned().into(),
			])
			.await;

			let _permit = HIDER.acquire().await.unwrap();
			defer!(AppProxy::resume());
			AppProxy::stop().await;

			let new: Vec<_> = fs::read_to_string(&tmp).await?.lines().map(PathBuf::from).collect();
			Self::bulk_rename_do(root, old, new).await
		});
	}

	async fn bulk_rename_do(root: PathBuf, old: Vec<PathBuf>, new: Vec<PathBuf>) -> Result<()> {
		terminal_clear(&mut stderr())?;
		if old.len() != new.len() {
			eprintln!("Number of old and new differ, press ENTER to exit");
			stdin().read_exact(&mut [0]).await?;
			return Ok(());
		}

		let (old, new) = old.into_iter().zip(new).filter(|(o, n)| o != n).unzip();
		let todo = Self::prioritized_paths(old, new);
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

			if maybe_exists(&new).await && !paths_to_same_file(&old, &new).await {
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
			Pubsub::pub_from_bulk(succeeded.iter().map(|(o, n)| (o, &n.url)).collect());
			FilesOp::rename(succeeded);
		}
		drop(permit);

		if !failed.is_empty() {
			Self::output_failed(failed).await?;
		}
		Ok(())
	}

	async fn output_failed(failed: Vec<(PathBuf, PathBuf, anyhow::Error)>) -> Result<()> {
		terminal_clear(&mut stderr())?;

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

	fn prioritized_paths(old: Vec<PathBuf>, new: Vec<PathBuf>) -> Vec<(PathBuf, PathBuf)> {
		let orders: HashMap<_, _> = old.iter().enumerate().map(|(i, p)| (p, i)).collect();
		let mut incomes: HashMap<_, _> = old.iter().map(|p| (p, false)).collect();
		let mut todos: HashMap<_, _> = old
			.iter()
			.zip(new)
			.map(|(o, n)| {
				incomes.get_mut(&n).map(|b| *b = true);
				(o, n)
			})
			.collect();

		let mut sorted = Vec::with_capacity(old.len());
		while !todos.is_empty() {
			// Paths that are non-incomes and don't need to be prioritized in this round
			let mut outcomes: Vec<_> = incomes.iter().filter(|(_, &b)| !b).map(|(&p, _)| p).collect();
			outcomes.sort_unstable_by(|a, b| orders[b].cmp(&orders[a]));

			// If there're no outcomes, it means there are cycles in the renaming
			if outcomes.is_empty() {
				let mut remain: Vec<_> = todos.into_iter().map(|(o, n)| (o.clone(), n)).collect();
				remain.sort_unstable_by(|(a, _), (b, _)| orders[a].cmp(&orders[b]));
				sorted.reverse();
				sorted.extend(remain);
				return sorted;
			}

			for old in outcomes {
				let Some(new) = todos.remove(old) else { unreachable!() };
				incomes.remove(&old);
				incomes.get_mut(&new).map(|b| *b = false);
				sorted.push((old.clone(), new));
			}
		}
		sorted.reverse();
		sorted
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_sort() {
		fn cmp(input: &[(&str, &str)], expected: &[(&str, &str)]) {
			let sorted = Mgr::prioritized_paths(
				input.iter().map(|&(o, _)| o.into()).collect(),
				input.iter().map(|&(_, n)| n.into()).collect(),
			);
			let sorted: Vec<_> =
				sorted.iter().map(|(o, n)| (o.to_str().unwrap(), n.to_str().unwrap())).collect();
			assert_eq!(sorted, expected);
		}

		#[rustfmt::skip]
		cmp(
			&[("2", "3"), ("1", "2"), ("3", "4")],
			&[("3", "4"), ("2", "3"), ("1", "2")]
		);

		#[rustfmt::skip]
		cmp(
			&[("1", "3"), ("2", "3"), ("3", "4")],
			&[("3", "4"), ("1", "3"), ("2", "3")]
		);

		#[rustfmt::skip]
		cmp(
			&[("2", "1"), ("1", "2")],
			&[("2", "1"), ("1", "2")]
		);

		#[rustfmt::skip]
		cmp(
			&[("3", "2"), ("2", "1"), ("1", "3"), ("a", "b"), ("b", "c")],
			&[("b", "c"), ("a", "b"), ("3", "2"), ("2", "1"), ("1", "3")]
		);

		#[rustfmt::skip]
		cmp(
			&[("b", "b_"), ("a", "a_"), ("c", "c_")],
			&[("b", "b_"), ("a", "a_"), ("c", "c_")],
		);
	}
}
