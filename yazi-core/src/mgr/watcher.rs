use std::{collections::{HashMap, HashSet}, time::Duration};

use anyhow::Result;
use notify::{PollWatcher, RecommendedWatcher, RecursiveMode, Watcher as _Watcher};
use parking_lot::RwLock;
use tokio::{fs, pin, sync::{mpsc::{self, UnboundedReceiver}, watch}};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tracing::error;
use yazi_fs::{File, Files, FilesOp, cha::Cha, realname_unchecked};
use yazi_proxy::WATCHER;
use yazi_shared::{RoCell, url::Url};

use super::Linked;
use crate::tab::Folder;

pub(crate) static WATCHED: RoCell<RwLock<HashSet<Url>>> = RoCell::new();
pub static LINKED: RoCell<RwLock<Linked>> = RoCell::new();

pub struct Watcher {
	in_tx:  watch::Sender<HashSet<Url>>,
	out_tx: mpsc::UnboundedSender<Url>,
}

impl Watcher {
	pub(super) fn serve() -> Self {
		let (in_tx, in_rx) = watch::channel(Default::default());
		let (out_tx, out_rx) = mpsc::unbounded_channel();

		let out_tx_ = out_tx.clone();
		let handler = move |res: Result<notify::Event, notify::Error>| {
			let Ok(event) = res else { return };
			if event.kind.is_access() {
				return;
			}
			Self::push_files_impl(&out_tx_, event.paths.into_iter().map(Url::from));
		};

		let config = notify::Config::default().with_poll_interval(Duration::from_millis(500));
		if yazi_adapter::WSL.get() {
			tokio::spawn(Self::fan_in(in_rx, PollWatcher::new(handler, config).unwrap()));
		} else {
			tokio::spawn(Self::fan_in(in_rx, RecommendedWatcher::new(handler, config).unwrap()));
		}

		#[cfg(any(target_os = "linux", target_os = "macos"))]
		yazi_fs::mounts::Partitions::monitor(
			yazi_fs::mounts::PARTITIONS.clone(),
			yazi_dds::Pubsub::pub_from_mount,
		);

		tokio::spawn(Self::fan_out(out_rx));
		Self { in_tx, out_tx }
	}

	pub(super) fn watch(&mut self, mut new: HashSet<&Url>) {
		new.retain(|&u| u.is_regular());
		self.in_tx.send(new.into_iter().cloned().collect()).ok();
	}

	pub(super) fn push_files(&self, urls: Vec<Url>) {
		Self::push_files_impl(&self.out_tx, urls.into_iter());
	}

	fn push_files_impl(out_tx: &mpsc::UnboundedSender<Url>, urls: impl Iterator<Item = Url>) {
		let (mut parents, watched) = (HashSet::new(), WATCHED.read());
		for u in urls {
			let Some(p) = u.parent_url() else { continue };
			if !watched.contains(&p) && !LINKED.read().from_dir(&p).any(|u| watched.contains(u)) {
				continue;
			}
			out_tx.send(u).ok();
			if !parents.contains(&p) {
				out_tx.send(p.clone()).ok();
				parents.insert(p);
			}
		}
	}

	// TODO: performance improvement
	pub(super) fn trigger_dirs(&self, folders: &[&Folder]) {
		let todo: Vec<_> =
			folders.iter().filter(|&f| f.url.is_regular()).map(|&f| (f.url.to_owned(), f.cha)).collect();
		if todo.is_empty() {
			return;
		}

		async fn go(cwd: Url, cha: Cha) {
			let Some(cha) = Files::assert_stale(&cwd, cha).await else { return };

			match Files::from_dir_bulk(&cwd).await {
				Ok(files) => FilesOp::Full(cwd, files, cha).emit(),
				Err(e) => FilesOp::issue_error(&cwd, e.kind()).await,
			}
		}

		tokio::spawn(async move {
			futures::future::join_all(todo.into_iter().map(|(cwd, cha)| go(cwd, cha))).await;
		});
	}

	async fn fan_in(
		mut rx: watch::Receiver<HashSet<Url>>,
		mut watcher: impl notify::Watcher + Send + 'static,
	) {
		loop {
			let (to_unwatch, to_watch): (HashSet<_>, HashSet<_>) = {
				let (new, old) = (&*rx.borrow_and_update(), &*WATCHED.read());
				(old.difference(new).cloned().collect(), new.difference(old).cloned().collect())
			};

			watcher = Self::sync_watched(watcher, to_unwatch, to_watch).await;

			if !rx.has_changed().unwrap_or(false) {
				Self::sync_linked().await;
			}

			if rx.changed().await.is_err() {
				break;
			}
		}
	}

	async fn fan_out(rx: UnboundedReceiver<Url>) {
		// TODO: revert this once a new notification is implemented
		let rx = UnboundedReceiverStream::new(rx).chunks_timeout(1000, Duration::from_millis(100));
		pin!(rx);

		while let Some(chunk) = rx.next().await {
			let urls: HashSet<_> = chunk.into_iter().collect();
			let mut cached: HashMap<_, _> = HashMap::new();

			let _permit = WATCHER.acquire().await.unwrap();
			let mut ops = Vec::with_capacity(urls.len());

			for u in urls {
				let Some((parent, urn)) = u.pair() else { continue };
				let Ok(file) = File::new(u).await else {
					ops.push(FilesOp::Deleting(parent, [urn].into()));
					continue;
				};

				let u = &file.url;
				let eq = (!file.is_link() && fs::canonicalize(u).await.is_ok_and(|p| p == ***u))
					|| realname_unchecked(u, &mut cached).await.is_ok_and(|s| urn.as_urn() == s);

				if !eq {
					ops.push(FilesOp::Deleting(parent, [urn].into()));
					continue;
				}

				ops.push(FilesOp::Upserting(parent, [(urn, file)].into()));
			}

			FilesOp::mutate(ops);
		}
	}

	async fn sync_watched<W>(mut watcher: W, to_unwatch: HashSet<Url>, to_watch: HashSet<Url>) -> W
	where
		W: notify::Watcher + Send + 'static,
	{
		use notify::ErrorKind::WatchNotFound;

		if to_unwatch.is_empty() && to_watch.is_empty() {
			return watcher;
		}

		tokio::task::spawn_blocking(move || {
			for u in to_unwatch {
				match watcher.unwatch(&u) {
					Ok(()) => _ = WATCHED.write().remove(&u),
					Err(e) if matches!(e.kind, WatchNotFound) => _ = WATCHED.write().remove(&u),
					Err(e) => error!("Unwatch failed: {e:?}"),
				}
			}
			for u in to_watch {
				if watcher.watch(&u, RecursiveMode::NonRecursive).is_ok() {
					WATCHED.write().insert(u);
				}
			}
			watcher
		})
		.await
		.unwrap()
	}

	async fn sync_linked() {
		let mut new = WATCHED.read().clone();

		let old = {
			let mut linked = LINKED.write();
			linked.retain(|k, _| new.remove(k));
			linked.keys().cloned().collect()
		};

		async fn go(todo: HashSet<Url>) {
			for from in todo {
				let Ok(to) = fs::canonicalize(&from).await else { continue };

				if to != **from && WATCHED.read().contains(&from) {
					LINKED.write().insert(from, Url::from(to));
				}
			}
		}

		go(new).await;
		go(old).await;
	}
}
