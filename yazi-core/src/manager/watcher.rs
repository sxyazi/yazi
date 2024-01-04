use std::{collections::{BTreeMap, BTreeSet}, sync::Arc, time::Duration};

use notify::{event::{MetadataKind, ModifyKind}, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _Watcher};
use parking_lot::RwLock;
use tokio::{fs, pin, sync::mpsc::{self, UnboundedReceiver}};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tracing::error;
use yazi_plugin::isolate;
use yazi_shared::fs::{File, FilesOp, Url};

use super::Linked;
use crate::folder::{Files, Folder};

pub struct Watcher {
	watcher:    RecommendedWatcher,
	watched:    Arc<RwLock<BTreeSet<Url>>>,
	pub linked: Arc<RwLock<Linked>>,
}

impl Watcher {
	pub(super) fn start() -> Self {
		let (tx, rx) = mpsc::unbounded_channel();
		let watcher = RecommendedWatcher::new(
			{
				let tx = tx.clone();
				move |res: Result<notify::Event, notify::Error>| {
					let Ok(event) = res else {
						return;
					};

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

	pub(super) fn watch(&mut self, mut new: BTreeSet<&Url>) {
		new.retain(|&u| u.is_regular());
		let (to_unwatch, to_watch): (BTreeSet<_>, BTreeSet<_>) = {
			let guard = self.watched.read();
			let old: BTreeSet<_> = guard.iter().collect();
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

		tokio::spawn(async move {
			for (url, mtime) in todo {
				let Ok(meta) = fs::metadata(&url).await else {
					if let Ok(m) = fs::symlink_metadata(&url).await {
						FilesOp::Full(url, vec![], m.modified().ok()).emit();
					} else if let Some(p) = url.parent_url() {
						FilesOp::Deleting(p, vec![url]).emit();
					}
					continue;
				};

				if meta.modified().ok() == mtime {
					continue;
				}

				if let Ok(rx) = Files::from_dir(&url).await {
					let files: Vec<_> = UnboundedReceiverStream::new(rx).collect().await;
					FilesOp::Full(url, files, meta.modified().ok()).emit();
				}
			}
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
		let rx = UnboundedReceiverStream::new(rx).chunks_timeout(10, Duration::from_millis(20));
		pin!(rx);

		while let Some(urls) = rx.next().await {
			let urls: BTreeSet<_> = urls.into_iter().collect();
			let mut reload = Vec::with_capacity(urls.len());

			for u in urls {
				let Some(parent) = u.parent_url() else {
					continue;
				};

				let Ok(file) = File::from(u.clone()).await else {
					FilesOp::Deleting(parent, vec![u]).emit();
					continue;
				};

				if !file.is_dir() {
					reload.push(file.clone());
				}
				FilesOp::Upserting(parent, BTreeMap::from_iter([(u, file)])).emit();
			}

			if reload.is_empty() {
				continue;
			}
			if let Err(e) = isolate::preload("mime.lua", reload, true).await {
				error!("preload in watcher failed: {e}");
			}
		}
	}
}
