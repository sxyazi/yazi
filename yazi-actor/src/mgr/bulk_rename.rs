use std::{
	hash::Hash,
	io::{Read, Write},
	ops::Deref,
	path::Path,
	sync::Arc,
};

use anyhow::{Result, anyhow};
use hashbrown::HashMap;
use scopeguard::defer;
use tokio::io::AsyncWriteExt;
use yazi_binding::Permit;
use yazi_config::{YAZI, opener::OpenerRule};
use yazi_dds::Pubsub;
use yazi_fs::{
	File, FilesOp, Splatter, max_common_root,
	path::skip_url,
	provider::{
		FileBuilder, Provider,
		local::{Gate, Local},
	},
};
use yazi_macro::{err, succ, writef};
use yazi_parser::VoidForm;
use yazi_proxy::TasksProxy;
use yazi_scheduler::{AppProxy, NotifyProxy};
use yazi_shared::{
	data::Data,
	path::PathDyn,
	strand::{AsStrand, AsStrandJoin, Strand, StrandBuf, StrandLike},
	timestamp_us,
	url::{AsUrl, UrlBuf, UrlCow, UrlLike},
};
use yazi_term::{YIELD_TO_SUBPROCESS, sequence::EraseScreen};
use yazi_tty::TTY;
use yazi_vfs::{VfsFile, maybe_exists, provider};
use yazi_watcher::WATCHER;

use crate::{Actor, Ctx};

type Renames = Vec<(Tuple, Tuple)>;

pub struct BulkRename;

impl Actor for BulkRename {
	type Form = VoidForm;

	const NAME: &str = "bulk_rename";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let Some(opener) = Self::opener() else {
			succ!(NotifyProxy::push_warn("Bulk rename", "No text opener found"));
		};

		let selected: Vec<_> = cx.tab().selected_or_hovered().cloned().collect();
		if selected.is_empty() {
			succ!(NotifyProxy::push_warn("Bulk rename", "No files selected"));
		}

		let root = max_common_root(&selected);
		let old: Vec<_> =
			selected.iter().enumerate().map(|(i, u)| Tuple::new(i, skip_url(u, root))).collect();

		let cwd = cx.cwd().clone();
		let batcher = cx.core.mgr.batcher.clone();
		tokio::spawn(async move {
			let tmp = YAZI.preview.tmpfile("bulk-rename");

			Gate::default()
				.write(true)
				.create_new(true)
				.open(&tmp)
				.await?
				.write_all(old.join(Strand::Utf8("\n")).encoded_bytes())
				.await?;

			defer! {
				let tmp = tmp.clone();
				batcher.drain(&tmp);
				tokio::spawn(async move {
					Local::regular(&tmp).remove_file().await
				});
			}

			batcher.prime(&tmp);
			TasksProxy::process_exec(
				cwd,
				Splatter::new(&[UrlCow::default(), tmp.as_url().into()]).splat(&opener.run),
				vec![UrlCow::default(), UrlBuf::from(&tmp).into()],
				opener.block,
				opener.orphan,
			)
			.await;

			let _permit = Permit::new(YIELD_TO_SUBPROCESS.acquire().await.unwrap(), AppProxy::resume());
			AppProxy::stop().await;

			let new: Vec<_> = Local::regular(&tmp)
				.read_to_string()
				.await?
				.lines()
				.take(old.len())
				.enumerate()
				.map(|(i, s)| Tuple::new(i, s))
				.collect();

			let decision = batcher.drain(&tmp);
			Self::r#do(root, old, new, selected, decision).await
		});
		succ!();
	}
}

impl BulkRename {
	async fn r#do(
		root: usize,
		old: Vec<Tuple>,
		new: Vec<Tuple>,
		selected: Vec<UrlBuf>,
		decision: Option<bool>,
	) -> Result<()> {
		writef!(TTY.writer(), "{EraseScreen}\n")?;
		if old.len() != new.len() {
			#[rustfmt::skip]
			writef!(TTY.writer(), "Number of new and old file names mismatch (New: {}, Old: {}).\nPress <Enter> to exit...", new.len(), old.len())?;

			TTY.reader().read_exact(&mut [0])?;
			return Ok(());
		}

		let (old, new) = old.into_iter().zip(new).filter(|(o, n)| o != n).unzip();
		let (chain, cycles) = Self::prioritized_paths(old, new);
		if chain.is_empty() && cycles.is_empty() {
			return Ok(());
		}

		let todo: Vec<_> = chain.iter().chain(cycles.iter().flatten()).cloned().collect();
		if !Self::ask_continue(&todo, decision)? {
			return Ok(());
		}

		let permit = WATCHER.acquire().await.unwrap();
		let cap = chain.len() + cycles.iter().map(Vec::len).sum::<usize>();
		let (mut failed, mut succeeded) = (Vec::new(), HashMap::with_capacity(cap));
		for (o, n) in chain {
			let (Ok(old), Ok(new)) =
				(Self::replace_url(&selected[o.0], root, &o), Self::replace_url(&selected[n.0], root, &n))
			else {
				failed.push((o, n, anyhow!("Invalid new or old file name")));
				continue;
			};

			if maybe_exists(&new).await && !provider::must_identical(&old, &new).await {
				failed.push((o, n, anyhow!("Destination already exists")));
			} else if let Err(e) = provider::rename(&old, &new).await {
				failed.push((o, n, e.into()));
			} else if let Ok(f) = File::new(new).await {
				succeeded.insert(old, f);
			} else {
				failed.push((o, n, anyhow!("Failed to retrieve file info")));
			}
		}

		for cycle in cycles {
			Self::rename_cycle(root, cycle, &selected, &mut failed, &mut succeeded).await;
		}

		if !succeeded.is_empty() {
			let it = succeeded.iter().map(|(o, n)| (o.as_url(), n.url.as_url()));
			err!(Pubsub::pub_after_bulk_rename(it));
			FilesOp::rename(succeeded);
		}
		drop(permit);

		if !failed.is_empty() {
			Self::output_failed(failed).await?;
		}
		Ok(())
	}

	fn opener() -> Option<Arc<OpenerRule>> {
		YAZI
			.open
			.match_dummy(Path::new("bulk-rename.txt"), "text/plain")
			.and_then(|r| YAZI.opener.block(&r))
	}

	fn replace_url(url: &UrlBuf, take: usize, rep: &StrandBuf) -> Result<UrlBuf> {
		Ok(url.try_replace(take, PathDyn::with(url.kind(), rep)?)?.into_owned())
	}

	async fn rename_cycle(
		root: usize,
		cycle: Renames,
		selected: &[UrlBuf],
		failed: &mut Vec<(Tuple, Tuple, anyhow::Error)>,
		succeeded: &mut HashMap<UrlBuf, File>,
	) {
		let edge = |i: usize, e: anyhow::Error| (cycle[i].0.clone(), cycle[i].1.clone(), e);

		let urls: Result<Vec<_>> = cycle
			.iter()
			.map(|(o, n)| {
				Ok((
					Self::replace_url(&selected[o.0], root, o)?,
					Self::replace_url(&selected[n.0], root, n)?,
				))
			})
			.collect();
		let Ok(urls) = urls else {
			failed.extend((0..cycle.len()).map(|i| edge(i, anyhow!("Invalid new or old file name"))));
			return;
		};

		let first = &urls[0].0;
		let Some(tmp) = Self::temp_url(first, &urls).await else {
			failed
				.extend((0..cycle.len()).map(|i| edge(i, anyhow!("Failed to allocate a temporary name"))));
			return;
		};

		if let Err(e) = provider::rename(first, &tmp).await {
			failed.push(edge(0, e.into()));
			failed.extend((1..cycle.len()).map(|i| edge(i, anyhow!("Skipped due to a broken cycle"))));
			return;
		}

		for i in (1..cycle.len()).rev() {
			let (old, new) = &urls[i];
			if let Err(e) = provider::rename(old, new).await {
				err!(provider::rename(&tmp, first).await);
				failed.extend((0..i).map(|j| edge(j, anyhow!("Rolled back due to a broken cycle"))));
				failed.push(edge(i, e.into()));
				return;
			}
			match File::new(new.clone()).await {
				Ok(f) => {
					succeeded.insert(old.clone(), f);
				}
				Err(_) => failed.push(edge(i, anyhow!("Failed to retrieve file info"))),
			}
		}

		let final_to = &urls[0].1;
		if let Err(e) = provider::rename(&tmp, final_to).await {
			failed.push(edge(0, anyhow!("{e}; the file is left at {}", tmp.display())));
		}
		match File::new(final_to.clone()).await {
			Ok(f) => {
				succeeded.insert(first.clone(), f);
			}
			Err(_) => failed.push(edge(0, anyhow!("Failed to retrieve file info"))),
		}
	}

	async fn temp_url(file: &UrlBuf, urls: &[(UrlBuf, UrlBuf)]) -> Option<UrlBuf> {
		let parent = file.parent()?;
		for _ in 0..16 {
			let name = format!(".bulk-rename-{}", timestamp_us());
			let Ok(tmp) = parent.try_join(name.as_str()) else { continue };
			if urls.iter().any(|(_, n)| *n == tmp) {
				continue;
			}
			if !maybe_exists(&tmp).await {
				return Some(tmp);
			}
		}
		None
	}

	fn ask_continue(todo: &[(Tuple, Tuple)], decision: Option<bool>) -> Result<bool> {
		if let Some(decision) = decision {
			return Ok(decision);
		}

		{
			let mut w = TTY.lockout();
			for (old, new) in todo {
				writeln!(w, "{} -> {}", old.display(), new.display())?;
			}
			write!(w, "Continue to rename? (y/N): ")?;
			w.flush()?;
		}

		let mut buf = [0; 10];
		_ = TTY.reader().read(&mut buf)?;
		Ok(buf[0] == b'y' || buf[0] == b'Y')
	}

	async fn output_failed(failed: Vec<(Tuple, Tuple, anyhow::Error)>) -> Result<()> {
		let mut stdout = TTY.lockout();
		writeln!(stdout, "{EraseScreen}")?;

		writeln!(stdout, "Failed to rename:")?;
		for (old, new, err) in failed {
			writeln!(stdout, "{} -> {}: {err}", old.display(), new.display())?;
		}
		writeln!(stdout, "\nPress ENTER to exit")?;

		stdout.flush()?;
		TTY.reader().read_exact(&mut [0])?;
		Ok(())
	}

	fn prioritized_paths(old: Vec<Tuple>, new: Vec<Tuple>) -> (Renames, Vec<Renames>) {
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

			// If there're no outcomes, every remaining edge belongs to a cycle
			if outcomes.is_empty() {
				sorted.reverse();
				return (sorted, Self::partition_cycles(todos, &orders));
			}

			for old in outcomes {
				let Some(new) = todos.remove(old) else { unreachable!() };
				incomes.remove(&old);
				incomes.get_mut(&new).map(|b| *b = false);
				sorted.push((old.clone(), new));
			}
		}
		sorted.reverse();
		(sorted, Vec::new())
	}

	fn partition_cycles(
		mut todos: HashMap<&Tuple, Tuple>,
		orders: &HashMap<&Tuple, usize>,
	) -> Vec<Renames> {
		let mut starts: Vec<_> = todos.keys().copied().collect();
		starts.sort_unstable_by(|a, b| orders[a].cmp(&orders[b]));

		let mut cycles = Vec::new();
		for start in starts {
			if !todos.contains_key(start) {
				continue;
			}

			let mut cycle = Vec::new();
			let mut cur = start;
			while let Some(next) = todos.remove(cur) {
				cycle.push((cur.clone(), next.clone()));
				let Some((&owner, _)) = todos.get_key_value(&next) else { break };
				cur = owner;
			}
			cycles.push(cycle);
		}
		cycles
	}
}

// --- Tuple
#[derive(Clone, Debug)]
struct Tuple(usize, StrandBuf);

impl Deref for Tuple {
	type Target = StrandBuf;

	fn deref(&self) -> &Self::Target {
		&self.1
	}
}

impl PartialEq for Tuple {
	fn eq(&self, other: &Self) -> bool {
		self.1 == other.1
	}
}

impl Eq for Tuple {}

impl Hash for Tuple {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.1.as_strand().encoded_bytes().hash(state);
	}
}

impl AsStrand for &Tuple {
	fn as_strand(&self) -> Strand<'_> {
		self.1.as_strand()
	}
}

impl Tuple {
	fn new(index: usize, inner: impl Into<StrandBuf>) -> Self {
		Self(index, inner.into())
	}
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
