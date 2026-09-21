use std::{io, mem, ops::Deref, time::Duration};

use hashbrown::{HashMap, hash_map::RawEntryMut};
use indexmap::IndexSet;
use tokio::{pin, sync::mpsc, task::JoinHandle};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_fs::{Entries, file::File, op::{FILES_TICKET, FilesOp}};
use yazi_shared::{id::Id, url::{UrlBuf, UrlLike, UrlMapExt}};
use yazi_vfs::VfsEntries;

#[derive(Clone)]
pub struct Refresher {
	tx: mpsc::UnboundedSender<Op>,
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
						for (_, entry) in entries.iter_mut().filter(|(u, _)| !u.auth().is_local()) {
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
				entries.retain(|url, _| files.contains(url));
				for file in files {
					entries.get_or_insert_with(file, Entry::new);
				}
			}
			Op::Load(file) => {
				let entry = match entries.raw_entry_mut().from_key(&file.url) {
					RawEntryMut::Occupied(oe) if oe.get().busy != Id::ZERO => {
						oe.into_mut().file = file;
						return;
					}
					RawEntryMut::Occupied(mut oe) => {
						oe.get_mut().file = file;
						oe.into_mut()
					}
					RawEntryMut::Vacant(ve) => ve.insert(file.to_url(), Entry::new(file)).1,
				};

				(entry.dirty, entry.report, entry.force, entry.stream) = (true, true, true, true);
				self.spawn(entry);
			}
			Op::Refresh { file, force } => {
				let entry = match entries.raw_entry_mut().from_key(&file.url) {
					RawEntryMut::Occupied(oe) if !force && oe.get().busy != Id::ZERO => {
						oe.into_mut().file = file;
						return;
					}
					RawEntryMut::Occupied(mut oe) => {
						oe.get_mut().file = file;
						oe.into_mut()
					}
					RawEntryMut::Vacant(ve) => ve.insert(file.to_url(), Entry::new(file)).1,
				};

				(entry.dirty, entry.report, entry.force, entry.stream) =
					(true, true, entry.force || force, false);
				self.spawn(entry);
			}
			Op::Done(mut prev, result) => {
				let Some(entry) = entries.get_mut(&prev.url) else { return };
				if entry.busy != prev.busy {
					return;
				}
				if result.is_err() {
					entry.force = true;
				}

				match result {
					Ok(RefreshResponse::Full(files)) => {
						entry.file = prev.file.clone();
						FilesOp::Full(mem::take(&mut prev.file), files).emit();
					}
					Ok(RefreshResponse::Part) => {
						entry.file = prev.file.clone();
						FilesOp::Done(mem::take(&mut prev.file), prev.busy).emit();
					}
					Ok(RefreshResponse::Skip) => {}
					Err(e) if e.kind() == io::ErrorKind::NotFound => {
						if let Some((t, n)) = prev.pair() {
							FilesOp::Delete(t.into(), [n.into()].into()).emit();
						} else if prev.report {
							FilesOp::Fail(mem::take(&mut prev.file.url), e.into()).emit();
						}
					}
					Err(e) if prev.report => {
						FilesOp::Fail(mem::take(&mut prev.file.url), e.into()).emit();
					}
					Err(e) => yazi_macro::debug!("Failed to refresh {}: {e:?}", prev.url),
				}

				entry.busy = Id::ZERO;
				self.spawn(entry); // A new request may have arrived while this entry was busy.
			}
		}
	}

	fn spawn(&self, entry: &mut Entry) {
		if !entry.dirty || entry.busy != Id::ZERO {
			return;
		}

		let (tx, mut prev) = (self.tx.clone(), entry.turn());
		entry.handle = Some(tokio::spawn(async move {
			let result = async {
				if let Some(file) = Entries::revalidate(&prev.file).await? {
					prev.file = file;
				} else if !prev.force {
					return Ok(RefreshResponse::Skip);
				}

				if prev.stream {
					Self::spawn_part(&mut prev).await
				} else {
					Self::spawn_full(&mut prev).await
				}
			}
			.await;
			tx.send(Op::Done(prev, result)).ok();
		}));
	}

	async fn spawn_full(prev: &mut Entry) -> io::Result<RefreshResponse> {
		Ok(RefreshResponse::Full(Entries::from_dir_bulk(&prev.url).await?))
	}

	async fn spawn_part(prev: &mut Entry) -> io::Result<RefreshResponse> {
		FilesOp::Part(prev.to_url(), vec![], prev.busy).emit();

		let rx = UnboundedReceiverStream::new(Entries::from_dir(&prev.url).await?)
			.chunks_timeout(5000, Duration::from_millis(500));
		pin!(rx);

		while let Some(chunk) = rx.next().await {
			FilesOp::Part(prev.to_url(), chunk, prev.busy).emit();
		}
		Ok(RefreshResponse::Part)
	}
}

impl Refresher {
	pub(super) fn sync(&self, files: IndexSet<File>) { self.tx.send(Op::Sync(files)).ok(); }

	pub fn load(&self, file: impl Into<File>) { self.tx.send(Op::Load(file.into())).ok(); }

	pub fn request<I>(&self, ops: I)
	where
		I: IntoIterator,
		I::Item: Into<Op>,
	{
		for op in ops {
			self.tx.send(op.into()).ok();
		}
	}

	pub fn shutdown(&self) { self.sync(IndexSet::new()); }
}

pub enum Op {
	Sync(IndexSet<File>),
	Load(File),
	Refresh { file: File, force: bool },
	Done(Entry, io::Result<RefreshResponse>),
}

impl Op {
	pub fn is_force(&self) -> bool { matches!(self, Self::Refresh { force: true, .. }) }
}

// --- Response
pub enum RefreshResponse {
	Full(Vec<File>),
	Part,
	Skip,
}

// --- Entry
#[derive(Default)]
pub struct Entry {
	file:   File,
	busy:   Id,
	dirty:  bool,
	report: bool,
	force:  bool,
	stream: bool,
	handle: Option<JoinHandle<()>>,
}

impl Deref for Entry {
	type Target = File;

	fn deref(&self) -> &Self::Target { &self.file }
}

impl Drop for Entry {
	fn drop(&mut self) { self.handle.take().map(|h| h.abort()); }
}

impl Entry {
	fn new(file: File) -> Self {
		let mut me = Self::default();
		me.file = file;
		me
	}

	fn turn(&mut self) -> Self {
		self.busy = FILES_TICKET.next();

		Self {
			file:   self.file.clone(),
			busy:   self.busy,
			dirty:  mem::take(&mut self.dirty),
			report: mem::take(&mut self.report),
			force:  mem::take(&mut self.force),
			stream: mem::take(&mut self.stream),
			handle: None,
		}
	}
}
