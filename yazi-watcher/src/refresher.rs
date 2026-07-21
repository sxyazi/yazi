use std::{io, ops::Deref, time::{Duration, Instant}};

use hashbrown::HashMap;
use indexmap::IndexSet;
use tokio::sync::mpsc;
use yazi_fs::{Entries, FilesOp, file::{File, FileCov}};
use yazi_shared::url::{UrlBuf, UrlCov, UrlLike, UrlMapExt};
use yazi_vfs::VfsEntries;

#[derive(Clone)]
pub struct Refresher {
	tx: mpsc::UnboundedSender<Op>,
}

enum Op {
	Sync(IndexSet<FileCov>),
	Refresh(IndexSet<FileCov>),
	Load(File),
	Touch(IndexSet<UrlBuf>),
	Done(Entry, io::Result<Option<Vec<File>>>),
}

impl Refresher {
	pub(super) fn serve() -> Self {
		let (tx, mut rx) = mpsc::unbounded_channel();
		let me = Self { tx };

		let me_ = me.clone();
		tokio::spawn(async move {
			let mut entries = HashMap::new();
			let mut interval = tokio::time::interval(Duration::from_secs(2));

			loop {
				tokio::select! {
					Some(op) = rx.recv() => me_.handle(op, &mut entries).await,
					_ = interval.tick() => {
						for (_, entry) in entries.iter_mut().filter(|(u, _)| u.kind().is_virtual()) {
							entry.dirty = true;
							me_.spawn(entry);
						}
					}
				}
			}
		});

		me
	}

	async fn handle(&self, op: Op, entries: &mut HashMap<UrlBuf, Entry>) {
		match op {
			Op::Sync(files) => {
				entries.retain(|url, _| files.contains(&UrlCov::new(url)));
				for file in files {
					entries.get_or_insert_with(file.0, |file| Entry { file, ..Default::default() });
				}
			}
			Op::Refresh(files) => {
				for file in files {
					let entry =
						entries.get_or_insert_with(file.0, |file| Entry { file, ..Default::default() });
					(entry.dirty, entry.report) = (true, true);
					self.spawn(entry);
				}
			}
			Op::Load(file) => {
				let entry = entries.get_or_insert_with(file, |file| Entry { file, ..Default::default() });
				(entry.dirty, entry.report, entry.force) = (true, true, true);
				self.spawn(entry);
			}
			Op::Touch(urls) => {
				for url in urls {
					let Some(entry) = entries.get_mut(&url) else { continue };
					entry.dirty = true;
					self.spawn(entry);
				}
			}
			Op::Done(prev, result) => {
				let Some(entry) = entries.get_mut(&prev.url) else { return };
				if entry.busy != prev.busy {
					return;
				}

				match result {
					Ok(Some(files)) => {
						entry.file = prev.file.clone();
						FilesOp::Full(prev.file, files).emit();
					}
					Ok(None) => {}
					Err(e) if e.kind() == io::ErrorKind::NotFound => {
						if let Some((p, n)) = prev.url.pair2() {
							FilesOp::Deleting(p.into(), [n.into()].into()).emit();
						}
					}
					Err(e) if prev.report => {
						FilesOp::IOErr(prev.file.url, e.into()).emit();
					}
					Err(e) => tracing::debug!("Failed to refresh {:?}: {e:?}", prev.url),
				}

				entry.busy = None;
			}
		}
	}

	fn spawn(&self, entry: &mut Entry) {
		if entry.busy.is_some() || !entry.dirty {
			return;
		}

		let (tx, mut prev) = (self.tx.clone(), entry.turn());
		tokio::spawn(async move {
			let result = async {
				Ok(if prev.force {
					Some(Entries::from_dir_bulk(&prev.file.url).await?)
				} else if let Some(file) = Entries::revalidate(&prev.file).await? {
					prev.file = file;
					Some(Entries::from_dir_bulk(&prev.url).await?)
				} else {
					None
				})
			}
			.await;
			tx.send(Op::Done(prev, result)).ok();
		});
	}
}

impl Refresher {
	pub(super) fn sync(&self, files: IndexSet<FileCov>) { self.tx.send(Op::Sync(files)).ok(); }

	pub fn refresh<I>(&self, files: I)
	where
		I: IntoIterator,
		I::Item: Into<File>,
	{
		let entries = files.into_iter().map(|file| FileCov(file.into())).collect();
		self.tx.send(Op::Refresh(entries)).ok();
	}

	pub fn load(&self, file: impl Into<File>) { self.tx.send(Op::Load(file.into())).ok(); }

	pub fn touch<I>(&self, urls: I)
	where
		I: IntoIterator<Item = UrlBuf>,
	{
		self.tx.send(Op::Touch(urls.into_iter().collect())).ok();
	}
}

// --- Entry
#[derive(Clone, Default)]
struct Entry {
	file:   File,
	busy:   Option<Instant>,
	dirty:  bool,
	report: bool,
	force:  bool,
}

impl Deref for Entry {
	type Target = File;

	fn deref(&self) -> &Self::Target { &self.file }
}

impl Entry {
	fn turn(&mut self) -> Self {
		self.busy = Some(Instant::now());
		let me = self.clone();

		(self.dirty, self.report, self.force) = (false, false, false);
		me
	}
}
