use std::{collections::{HashMap, HashSet}, sync::Arc, time::{Duration, SystemTime}};

use anyhow::Result;
use notify::{event::{MetadataKind, ModifyKind}, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _Watcher};
use parking_lot::RwLock;
use tokio::{fs, pin, sync::mpsc::{self, UnboundedReceiver}};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tracing::error;
use yazi_plugin::isolate;
use yazi_scheduler::WATCHER;
use yazi_shared::fs::{File, FilesOp, Url};

use super::Linked;
use crate::folder::{Files, Folder};

pub struct Watcher {
	watcher:    RecommendedWatcher,
	watched:    Arc<RwLock<HashSet<Url>>>,
	pub linked: Arc<RwLock<Linked>>,
}

impl Watcher {
	pub(super) fn start() -> Self {
		let (tx, rx) = mpsc::unbounded_channel();
		let watcher = RecommendedWatcher::new(
			{
				let tx = tx.clone();
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
						tx.send(Url::from(path)).ok();
					}
				}
			},
			Default::default(),
		);

		let instance =
			Self { watcher: watcher.unwrap(), watched: Default::default(), linked: Default::default() };
		tokio::spawn(Self::on_changed(rx));
		instance
	}

	pub(super) fn watch(&mut self, mut new: HashSet<&Url>) {
		new.retain(|&u| u.is_regular());
		let (to_unwatch, to_watch): (HashSet<_>, HashSet<_>) = {
			let guard = self.watched.read();
			let old: HashSet<_> = guard.iter().collect();
			(
				old.difference(&new).map(|&x| x.clone()).collect(),
				new.difference(&old).map(|&x| x.clone()).collect(),
			)
		};

		for u in to_unwatch {
			self.watcher.unwatch(&u).ok();
		}
		for u in to_watch {
			if self.watcher.watch(&u, RecursiveMode::NonRecursive).is_err() {
				new.remove(&u);
			}
		}

		*self.watched.write() = new.into_iter().cloned().collect();
		self.sync_linked();
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

	fn sync_linked(&self) {
		let mut new = self.watched.read().clone();
		self.linked.write().retain(|k, _| new.remove(k));

		let watched = self.watched.clone();
		let linked = self.linked.clone();
		macro_rules! go {
			($todo:expr) => {
				for from in $todo {
					match fs::canonicalize(&from).await {
						Ok(to) if to != *from && watched.read().contains(&from) => {
							linked.write().insert(from, Url::from(to));
						}
						_ => {}
					}
				}
			};
		}

		tokio::spawn(async move {
			let old: Vec<_> = linked.read().keys().cloned().collect();
			go!(new);
			go!(old);
		});
	}

	async fn on_changed(rx: UnboundedReceiver<Url>) {
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
}
