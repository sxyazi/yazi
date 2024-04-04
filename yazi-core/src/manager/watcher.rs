use std::{collections::{HashMap, HashSet}, time::{Duration, SystemTime}};

use anyhow::Result;
use notify::{event::{MetadataKind, ModifyKind}, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _Watcher};
use parking_lot::RwLock;
use tokio::{fs, pin, sync::{mpsc::{self, UnboundedReceiver}, watch}};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tracing::error;
use yazi_plugin::isolate;
use yazi_proxy::WATCHER;
use yazi_shared::{fs::{File, FilesOp, Url}, RoCell};

use super::Linked;
use crate::folder::{Files, Folder};

pub(crate) static WATCHED: RoCell<RwLock<HashSet<Url>>> = RoCell::new();
pub static LINKED: RoCell<RwLock<Linked>> = RoCell::new();

pub struct Watcher {
	tx: watch::Sender<(HashSet<Url>, HashSet<Url>)>,
}

impl Watcher {
	pub(super) fn serve() -> Self {
		let (in_tx, in_rx) = watch::channel(Default::default());
		let (out_tx, out_rx) = mpsc::unbounded_channel();

		let watcher = RecommendedWatcher::new(
			move |res: Result<notify::Event, notify::Error>| {
				let Ok(event) = res else { return };

				match event.kind {
					EventKind::Create(_) => {}
					EventKind::Modify(kind) => match kind {
						ModifyKind::Data(_) => {}
						ModifyKind::Metadata(md) => match md {
							MetadataKind::WriteTime => {}
							MetadataKind::Permissions => {}
							MetadataKind::Ownership => {}
							_ => return,
						},
						ModifyKind::Name(_) => {}
						_ => return,
					},
					EventKind::Remove(_) => {}
					_ => return,
				}

				for path in event.paths {
					out_tx.send(Url::from(path)).ok();
				}
			},
			Default::default(),
		);

		tokio::spawn(Self::on_in(in_rx, watcher.unwrap()));
		tokio::spawn(Self::on_out(out_rx));
		Self { tx: in_tx }
	}

	pub(super) fn watch(&mut self, mut new: HashSet<&Url>) {
		new.retain(|&u| u.is_regular());

		let old = WATCHED.read();
		let old: HashSet<_> = old.iter().collect();

		let (to_unwatch, to_watch): (HashSet<_>, HashSet<_>) = (
			old.difference(&new).map(|&x| x.clone()).collect(),
			new.difference(&old).map(|&x| x.clone()).collect(),
		);

		self.tx.send((to_unwatch, to_watch)).ok();
	}

	pub(super) fn trigger_dirs(&self, folders: &[&Folder]) {
		let todo: Vec<_> =
			folders.iter().filter(|&f| f.cwd.is_regular()).map(|&f| (f.cwd.clone(), f.mtime)).collect();
		if todo.is_empty() {
			return;
		}

		async fn go(url: Url, mtime: Option<SystemTime>) {
			let Ok(meta) = fs::metadata(&url).await else {
				if let Ok(m) = fs::symlink_metadata(&url).await {
					FilesOp::Full(url, vec![], m.modified().ok()).emit();
				} else if let Some(p) = url.parent_url() {
					FilesOp::Deleting(p, vec![url]).emit();
				}
				return;
			};

			if meta.modified().ok() == mtime {
				return;
			}

			if let Ok(files) = Files::from_dir_bulk(&url).await {
				FilesOp::Full(url, files, meta.modified().ok()).emit();
			}
		}

		tokio::spawn(async move {
			futures::future::join_all(todo.into_iter().map(|(url, mtime)| go(url, mtime))).await;
		});
	}

	async fn on_in(
		mut rx: watch::Receiver<(HashSet<Url>, HashSet<Url>)>,
		mut watcher: RecommendedWatcher,
	) {
		loop {
			{
				let (ref to_unwatch, ref to_watch) = *rx.borrow_and_update();
				for u in to_unwatch {
					if watcher.unwatch(u).is_ok() {
						WATCHED.write().remove(u);
					}
				}
				for u in to_watch {
					if watcher.watch(u, RecursiveMode::NonRecursive).is_ok() {
						WATCHED.write().insert(u.clone());
					}
				}
			}

			if !rx.has_changed().unwrap_or(false) {
				Self::sync_linked().await;
			}

			if rx.changed().await.is_err() {
				break;
			}
		}
	}

	async fn on_out(rx: UnboundedReceiver<Url>) {
		// TODO: revert this once a new notification is implemented
		let rx = UnboundedReceiverStream::new(rx).chunks_timeout(1000, Duration::from_millis(50));
		pin!(rx);

		while let Some(urls) = rx.next().await {
			let _permit = WATCHER.acquire().await.unwrap();
			let mut reload = Vec::with_capacity(urls.len());

			for u in urls.into_iter().collect::<HashSet<_>>() {
				let Some(parent) = u.parent_url() else { continue };

				let Ok(file) = File::from(u.clone()).await else {
					FilesOp::Deleting(parent, vec![u]).emit();
					continue;
				};

				if !file.is_dir() {
					reload.push(file.clone());
				}
				FilesOp::Upserting(parent, HashMap::from_iter([(u, file)])).emit();
			}

			if reload.is_empty() {
				continue;
			}
			if let Err(e) = isolate::preload("mime", reload, true).await {
				error!("preload in watcher failed: {e}");
			}
		}
	}

	async fn sync_linked() {
		let mut new = WATCHED.read().clone();
		LINKED.write().retain(|k, _| new.remove(k));

		macro_rules! go {
			($todo:expr) => {
				for from in $todo {
					match fs::canonicalize(&from).await {
						Ok(to) if to != *from && WATCHED.read().contains(&from) => {
							LINKED.write().insert(from, Url::from(to));
						}
						_ => {}
					}
				}
			};
		}

		let old: Vec<_> = LINKED.read().keys().cloned().collect();
		go!(new);
		go!(old);
	}
}
