use std::{collections::{HashMap, HashSet}, time::Duration};

use anyhow::Result;
use notify::{PollWatcher, RecommendedWatcher, RecursiveMode, Watcher as _Watcher};
use parking_lot::RwLock;
use tokio::{fs, pin, sync::{mpsc::{self, UnboundedReceiver}, watch}};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tracing::error;
use yazi_fs::{Files, Folder};
use yazi_plugin::isolate;
use yazi_proxy::WATCHER;
use yazi_shared::{RoCell, event::Cmd, fs::{Cha, File, FilesOp, Url, realname_unchecked}};

use super::Linked;

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
			for path in event.paths {
				out_tx_.send(Url::from(path)).ok();
			}
		};

		let config = notify::Config::default().with_poll_interval(Duration::from_millis(500));
		if *yazi_adapter::WSL {
			tokio::spawn(Self::fan_in(in_rx, PollWatcher::new(handler, config).unwrap()));
		} else {
			tokio::spawn(Self::fan_in(in_rx, RecommendedWatcher::new(handler, config).unwrap()));
		}

		tokio::spawn(Self::fan_out(out_rx));
		Self { in_tx, out_tx }
	}

	pub(super) fn watch(&mut self, mut new: HashSet<&Url>) {
		new.retain(|&u| u.is_regular());
		self.in_tx.send(new.into_iter().cloned().collect()).ok();
	}

	pub(super) fn push_files(&self, url: Vec<Url>) {
		let watched = WATCHED.read();
		for u in url {
			if u.parent_url().is_some_and(|p| watched.contains(&p)) {
				self.out_tx.send(u).ok();
			}
		}
	}

	pub(super) fn trigger_dirs(&self, folders: &[&Folder]) {
		let todo: Vec<_> =
			folders.iter().filter(|&f| f.url.is_regular()).map(|&f| (f.url.to_owned(), f.cha)).collect();
		if todo.is_empty() {
			return;
		}

		async fn go(cwd: Url, cha: Cha) {
			let Some(cha) = Files::assert_stale(&cwd, cha).await else { return };

			if let Ok(files) = Files::from_dir_bulk(&cwd).await {
				FilesOp::Full(cwd, files, cha).emit();
			}
		}

		tokio::spawn(async move {
			futures::future::join_all(todo.into_iter().map(|(cwd, cha)| go(cwd, cha))).await;
		});
	}

	async fn fan_in(mut rx: watch::Receiver<HashSet<Url>>, mut watcher: impl notify::Watcher) {
		loop {
			let (mut to_unwatch, mut to_watch): (HashSet<_>, HashSet<_>) = {
				let (new, old) = (&*rx.borrow_and_update(), &*WATCHED.read());
				(old.difference(new).cloned().collect(), new.difference(old).cloned().collect())
			};

			to_unwatch.retain(|u| match watcher.unwatch(u) {
				Ok(_) => true,
				Err(e) if matches!(e.kind, notify::ErrorKind::WatchNotFound) => true,
				Err(e) => {
					error!("Unwatch failed: {e:?}");
					false
				}
			});
			to_watch.retain(|u| watcher.watch(u, RecursiveMode::NonRecursive).is_ok());

			{
				let mut watched = WATCHED.write();
				watched.retain(|u| !to_unwatch.contains(u));
				watched.extend(to_watch);
			}

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
			let mut reload = Vec::with_capacity(urls.len());

			for u in urls {
				let Some((parent, urn)) = u.pair() else { continue };
				let Ok(file) = File::from(u).await else {
					ops.push(FilesOp::Deleting(parent, HashSet::from_iter([urn])));
					continue;
				};

				let u = &file.url;
				let eq = (!file.is_link() && fs::canonicalize(u).await.is_ok_and(|p| p == ***u))
					|| realname_unchecked(u, &mut cached).await.is_ok_and(|s| urn.as_urn() == s);

				if !eq {
					ops.push(FilesOp::Deleting(parent, HashSet::from_iter([urn])));
					continue;
				}

				if !file.is_dir() {
					reload.push(file.clone());
				}
				ops.push(FilesOp::Upserting(parent, HashMap::from_iter([(urn, file)])));
			}

			FilesOp::mutate(ops);
			if let Err(e) = isolate::fetch(Cmd::new("mime").into(), reload).await {
				error!("Fetch `mime` failed in watcher: {e}");
			}
		}
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
