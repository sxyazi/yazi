use std::{hash::{Hash, Hasher}, io, mem, ops::Deref, time::Duration};

use hashbrown::{HashMap, hash_map::RawEntryMut};
use indexmap::{IndexSet, set::MutableValues};
use tokio::{pin, sync::mpsc, task::JoinHandle};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_fs::{Entries, FILES_TICKET, FilesOp, file::File};
use yazi_shared::{id::Id, url::{UrlBuf, UrlLike, UrlMapExt}};
use yazi_vfs::VfsEntries;

#[derive(Clone)]
pub struct Refresher {
	tx: mpsc::UnboundedSender<Op>,
}

enum Op {
	Sync(IndexSet<File>),
	Refresh(IndexSet<RefreshRequest>),
	Done(Entry, io::Result<RefreshResponse>),
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
			Op::Refresh(requests) => {
				for r @ RefreshRequest { force, stream, .. } in requests {
					let entry = match entries.raw_entry_mut().from_key(&r.url) {
						RawEntryMut::Occupied(oe) if oe.get().skip(&r) => {
							oe.into_mut().file = r.file;
							continue;
						}
						RawEntryMut::Occupied(mut oe) => {
							oe.get_mut().file = r.file;
							oe.into_mut()
						}
						RawEntryMut::Vacant(ve) => ve.insert(r.url.clone(), Entry::new(r.file)).1,
					};

					(entry.dirty, entry.report, entry.force, entry.stream) =
						(true, true, entry.force || force, entry.stream || stream);
					self.spawn(entry);
				}
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
						if let Some((t, n)) = prev.url.pair() {
							FilesOp::Deleting(t.into(), [n.into()].into()).emit();
						}
					}
					Err(e) if prev.report => {
						FilesOp::IOErr(mem::take(&mut prev.file.url), e.into()).emit();
					}
					Err(e) => yazi_macro::debug!("Failed to refresh {}: {e:?}", prev.url),
				}

				entry.busy = Id::ZERO;
				self.spawn(entry); // A new request may have arrived while this entry was busy.
			}
		}
	}

	fn spawn(&self, entry: &mut Entry) {
		if entry.busy != Id::ZERO || !entry.dirty {
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
		FilesOp::Part(prev.url.clone(), vec![], prev.busy).emit();

		let rx = UnboundedReceiverStream::new(Entries::from_dir(&prev.url).await?)
			.chunks_timeout(5000, Duration::from_millis(500));
		pin!(rx);

		while let Some(chunk) = rx.next().await {
			FilesOp::Part(prev.url.clone(), chunk, prev.busy).emit();
		}
		Ok(RefreshResponse::Part)
	}
}

impl Refresher {
	pub(super) fn sync(&self, files: IndexSet<File>) { self.tx.send(Op::Sync(files)).ok(); }

	pub fn refresh<I>(&self, requests: I)
	where
		I: IntoIterator,
		I::Item: Into<RefreshRequest>,
	{
		let mut pending = IndexSet::<RefreshRequest>::new();
		for request in requests.into_iter().map(Into::into) {
			if let Some((_, entry)) = pending.get_full_mut2(&request) {
				entry.merge(request);
			} else {
				pending.insert(request);
			}
		}
		self.tx.send(Op::Refresh(pending)).ok();
	}

	pub fn shutdown(&self) { self.sync(IndexSet::new()); }
}

// --- RefreshRequest
pub struct RefreshRequest {
	pub file:   File,
	pub force:  bool,
	pub stream: bool,
	pub ticket: Id,
}

impl Deref for RefreshRequest {
	type Target = File;

	fn deref(&self) -> &Self::Target { &self.file }
}

impl PartialEq for RefreshRequest {
	fn eq(&self, other: &Self) -> bool { self.file.url == other.file.url }
}

impl Eq for RefreshRequest {}

impl Hash for RefreshRequest {
	fn hash<H: Hasher>(&self, state: &mut H) { self.file.url.hash(state); }
}

impl RefreshRequest {
	pub fn force(file: impl Into<File>) -> Self {
		Self { file: file.into(), force: true, stream: true, ticket: Id::ZERO }
	}

	fn merge(&mut self, other: Self) {
		self.file = other.file;
		self.force |= other.force;
		self.stream |= other.stream;
		self.ticket = if other.stream { other.ticket } else { self.ticket };
	}
}

// --- Response
enum RefreshResponse {
	Full(Vec<File>),
	Part,
	Skip,
}

// --- Entry
#[derive(Default)]
struct Entry {
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

	fn skip(&self, request: &RefreshRequest) -> bool {
		if self.busy == Id::ZERO || request.force {
			return false;
		}

		!request.stream || self.busy == request.ticket
	}
}
