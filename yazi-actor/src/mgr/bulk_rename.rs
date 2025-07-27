use std::{borrow::Cow, collections::HashMap, ffi::{OsStr, OsString}, hash::Hash, io::{Read, Write}, ops::Deref};

use anyhow::{Result, anyhow};
use crossterm::{execute, style::Print};
use scopeguard::defer;
use tokio::io::AsyncWriteExt;
use yazi_config::YAZI;
use yazi_dds::Pubsub;
use yazi_fs::{File, FilesOp, max_common_root, maybe_exists, paths_to_same_file, services::{self, Local}, skip_url};
use yazi_macro::{err, succ};
use yazi_parser::VoidOpt;
use yazi_proxy::{AppProxy, HIDER, TasksProxy, WATCHER};
use yazi_shared::{OsStrJoin, event::Data, terminal_clear, url::{Component, Url}};
use yazi_term::tty::TTY;

use crate::{Actor, Ctx};

pub struct BulkRename;

impl Actor for BulkRename {
	type Options = VoidOpt;

	const NAME: &str = "bulk_rename";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let Some(opener) = YAZI.opener.block(YAZI.open.all("bulk-rename.txt", "text/plain")) else {
			succ!(AppProxy::notify_warn("Bulk rename", "No text opener found"));
		};

		let selected: Vec<_> = cx.tab().selected_or_hovered().cloned().collect();
		if selected.is_empty() {
			succ!(AppProxy::notify_warn("Bulk rename", "No files selected"));
		}

		let root = max_common_root(&selected);
		let old: Vec<_> =
			selected.iter().enumerate().map(|(i, u)| Tuple::new(i, skip_url(u, root))).collect();

		let cwd = cx.cwd().clone();
		tokio::spawn(async move {
			let tmp = YAZI.preview.tmpfile("bulk");
			// TODO: pull `OpenOptions` into `yazi_fs`
			tokio::fs::OpenOptions::new()
				.write(true)
				.create_new(true)
				.open(&tmp)
				.await?
				.write_all(old.join(OsStr::new("\n")).as_encoded_bytes())
				.await?;

			defer! { tokio::spawn(Local::remove_file(tmp.clone())); }
			TasksProxy::process_exec(Cow::Borrowed(opener), cwd, vec![
				OsString::new(),
				tmp.to_owned().into(),
			])
			.await;

			let _permit = HIDER.acquire().await.unwrap();
			defer!(AppProxy::resume());
			AppProxy::stop().await;

			let new: Vec<_> = Local::read_to_string(&tmp)
				.await?
				.lines()
				.take(old.len())
				.enumerate()
				.map(|(i, s)| Tuple::new(i, s))
				.collect();

			Self::r#do(root, old, new, selected).await
		});
		succ!();
	}
}

impl BulkRename {
	async fn r#do(root: usize, old: Vec<Tuple>, new: Vec<Tuple>, selected: Vec<Url>) -> Result<()> {
		terminal_clear(TTY.writer())?;
		if old.len() != new.len() {
			#[rustfmt::skip]
			let s = format!("Number of new and old file names mismatch (New: {}, Old: {}).\nPress <Enter> to exit...", new.len(), old.len());
			execute!(TTY.writer(), Print(s))?;

			TTY.reader().read_exact(&mut [0])?;
			return Ok(());
		}

		let (old, new) = old.into_iter().zip(new).filter(|(o, n)| o != n).unzip();
		let todo = Self::prioritized_paths(old, new);
		if todo.is_empty() {
			return Ok(());
		}

		{
			let mut w = TTY.lockout();
			for (old, new) in &todo {
				writeln!(w, "{} -> {}", old.display(), new.display())?;
			}
			write!(w, "Continue to rename? (y/N): ")?;
			w.flush()?;
		}

		let mut buf = [0; 10];
		_ = TTY.reader().read(&mut buf)?;
		if buf[0] != b'y' && buf[0] != b'Y' {
			return Ok(());
		}

		let permit = WATCHER.acquire().await.unwrap();
		let (mut failed, mut succeeded) = (Vec::new(), HashMap::with_capacity(todo.len()));
		for (o, n) in todo {
			let (old, new): (Url, Url) = (
				selected[o.0].components().take(root).chain([Component::Normal(&o)]).collect(),
				selected[n.0].components().take(root).chain([Component::Normal(&n)]).collect(),
			);

			if maybe_exists(&new).await && !paths_to_same_file(&old, &new).await {
				failed.push((o, n, anyhow!("Destination already exists")));
			} else if let Err(e) = services::rename(&old, &new).await {
				failed.push((o, n, e.into()));
			} else if let Ok(f) = File::new(new).await {
				succeeded.insert(old, f);
			} else {
				failed.push((o, n, anyhow!("Failed to retrieve file info")));
			}
		}

		if !succeeded.is_empty() {
			let it = succeeded.iter().map(|(o, n)| (o, &n.url));
			err!(Pubsub::pub_after_bulk(it));
			FilesOp::rename(succeeded);
		}
		drop(permit);

		if !failed.is_empty() {
			Self::output_failed(failed).await?;
		}
		Ok(())
	}

	async fn output_failed(failed: Vec<(Tuple, Tuple, anyhow::Error)>) -> Result<()> {
		let mut stdout = TTY.lockout();
		terminal_clear(&mut *stdout)?;

		writeln!(stdout, "Failed to rename:")?;
		for (old, new, err) in failed {
			writeln!(stdout, "{} -> {}: {err}", old.display(), new.display())?;
		}
		writeln!(stdout, "\nPress ENTER to exit")?;

		stdout.flush()?;
		TTY.reader().read_exact(&mut [0])?;
		Ok(())
	}

	fn prioritized_paths(old: Vec<Tuple>, new: Vec<Tuple>) -> Vec<(Tuple, Tuple)> {
		let orders: HashMap<_, _> = old.iter().enumerate().map(|(i, t)| (t, i)).collect();
		let mut incomes: HashMap<_, _> = old.iter().map(|t| (t, false)).collect();
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
			let mut outcomes: Vec<_> = incomes.iter().filter(|&(_, b)| !b).map(|(&t, _)| t).collect();
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

// --- Tuple
#[derive(Clone, Debug)]
struct Tuple(usize, OsString);

impl Deref for Tuple {
	type Target = OsStr;

	fn deref(&self) -> &Self::Target { &self.1 }
}

impl PartialEq for Tuple {
	fn eq(&self, other: &Self) -> bool { self.1 == other.1 }
}

impl Eq for Tuple {}

impl Hash for Tuple {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.1.hash(state); }
}

impl AsRef<OsStr> for Tuple {
	fn as_ref(&self) -> &OsStr { &self.1 }
}

impl Tuple {
	fn new(index: usize, inner: impl Into<OsString>) -> Self { Self(index, inner.into()) }
}

// --- Tests
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_sort() {
		fn cmp(input: &[(&str, &str)], expected: &[(&str, &str)]) {
			let sorted = BulkRename::prioritized_paths(
				input.iter().map(|&(o, _)| Tuple::new(0, o)).collect(),
				input.iter().map(|&(_, n)| Tuple::new(0, n)).collect(),
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
