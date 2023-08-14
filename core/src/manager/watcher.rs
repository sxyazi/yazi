use std::{collections::{BTreeMap, BTreeSet}, path::{Path, PathBuf}, sync::Arc};

use indexmap::IndexMap;
use notify::{event::{MetadataKind, ModifyKind}, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _Watcher};
use parking_lot::RwLock;
use tokio::{fs, sync::mpsc::{self, Receiver, Sender}};

use crate::{emit, files::{Files, FilesOp}};

pub struct Watcher {
	tx: Sender<PathBuf>,

	watcher: RecommendedWatcher,
	watched: Arc<RwLock<IndexMap<PathBuf, Option<PathBuf>>>>,
}

impl Watcher {
	pub(super) fn start() -> Self {
		let (tx, rx) = mpsc::channel(50);
		let watcher = RecommendedWatcher::new(
			{
				let tx = tx.clone();
				move |res: Result<notify::Event, notify::Error>| {
					let Ok(event) = res else {
						return;
					};

					let Some(path) = event.paths.first().cloned() else {
						return;
					};

					let parent = path.parent().unwrap_or(&path).to_path_buf();
					match event.kind {
						EventKind::Create(_) => {
							tx.blocking_send(parent).ok();
						}
						EventKind::Modify(kind) => {
							match kind {
								ModifyKind::Metadata(kind) => match kind {
									MetadataKind::Permissions => {}
									MetadataKind::Ownership => {}
									MetadataKind::Extended => {}
									_ => return,
								},
								ModifyKind::Name(_) => {}
								_ => return,
							};

							tx.blocking_send(path).ok();
							tx.blocking_send(parent).ok();
						}
						EventKind::Remove(_) => {
							tx.blocking_send(path).ok();
							tx.blocking_send(parent).ok();
						}
						_ => (),
					}
				}
			},
			Default::default(),
		);

		let instance = Self { tx, watcher: watcher.unwrap(), watched: Default::default() };
		tokio::spawn(Self::changed(rx, instance.watched.clone()));
		instance
	}

	async fn changed(
		mut rx: Receiver<PathBuf>,
		watched: Arc<RwLock<IndexMap<PathBuf, Option<PathBuf>>>>,
	) {
		while let Some(path) = rx.recv().await {
			let linked = watched
				.read()
				.iter()
				.map_while(|(k, v)| v.as_ref().and_then(|v| path.strip_prefix(v).ok()).map(|v| k.join(v)))
				.collect::<Vec<_>>();

			let result = Files::read_dir(&path).await;
			if linked.is_empty() {
				emit!(Files(match result {
					Ok(items) => FilesOp::Read(path, items),
					Err(_) => FilesOp::IOErr(path),
				}));
				continue;
			}

			for ori in linked {
				emit!(Files(match &result {
					Ok(items) => {
						let files = BTreeMap::from_iter(items.iter().map(|(p, f)| {
							let p = ori.join(p.strip_prefix(&path).unwrap());
							let f = f.clone().set_path(&p);
							(p, f)
						}));
						FilesOp::Read(ori, files)
					}
					Err(_) => FilesOp::IOErr(ori),
				}));
			}
		}
	}

	pub(super) fn watch(&mut self, mut to_watch: BTreeSet<PathBuf>) {
		let keys = self.watched.read().keys().cloned().collect::<BTreeSet<_>>();
		for p in keys.difference(&to_watch) {
			self.watcher.unwatch(p).ok();
		}
		for p in to_watch.clone().difference(&keys) {
			if self.watcher.watch(p, RecursiveMode::NonRecursive).is_err() {
				to_watch.remove(p);
			}
		}

		let mut todo = Vec::new();
		let mut watched = self.watched.write();
		*watched = IndexMap::from_iter(to_watch.into_iter().map(|k| {
			if let Some(v) = watched.remove(&k) {
				(k, v)
			} else {
				todo.push(k.clone());
				(k, None)
			}
		}));
		watched.sort_unstable_by(|_, a, _, b| b.cmp(a));

		let watched = self.watched.clone();
		tokio::spawn(async move {
			let mut ext = IndexMap::new();
			for k in todo {
				match fs::canonicalize(&k).await {
					Ok(v) if v != k => {
						ext.insert(k, Some(v));
					}
					_ => {}
				}
			}

			let mut watched = watched.write();
			watched.extend(ext);
			watched.sort_unstable_by(|_, a, _, b| b.cmp(a));
		});
	}

	pub(super) fn trigger(&self, path: &Path) {
		let tx = self.tx.clone();
		let path = path.to_path_buf();
		tokio::spawn(async move {
			tx.send(path).await.ok();
		});
	}
}
