use std::{collections::BTreeSet, sync::Arc, time::Duration};

use futures::StreamExt;
use indexmap::IndexMap;
use notify::{event::{MetadataKind, ModifyKind}, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _Watcher};
use parking_lot::RwLock;
use shared::{StreamBuf, Url};
use tokio::{fs, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{emit, external, files::{Files, FilesOp}};

pub struct Watcher {
	watcher: RecommendedWatcher,
	watched: Arc<RwLock<IndexMap<Url, Option<Url>>>>,
}

impl Watcher {
	pub(super) fn start() -> Self {
		let (tx, rx) = mpsc::unbounded_channel();
		let rx = StreamBuf::new(UnboundedReceiverStream::new(rx), Duration::from_millis(300));

		let watcher = RecommendedWatcher::new(
			{
				let tx = tx.clone();
				move |res: Result<notify::Event, notify::Error>| {
					let Ok(event) = res else {
						return;
					};

					let Some(path) = event.paths.first().map(Url::from) else {
						return;
					};

					let parent = path.parent_url().unwrap_or_else(|| path.clone());
					match event.kind {
						EventKind::Create(_) => {
							tx.send(parent).ok();
						}
						EventKind::Modify(kind) => {
							match kind {
								ModifyKind::Data(_) => {}
								ModifyKind::Metadata(kind) => match kind {
									MetadataKind::Permissions => {}
									MetadataKind::Ownership => {}
									MetadataKind::Extended => {}
									_ => return,
								},
								ModifyKind::Name(_) => {}
								_ => return,
							};

							tx.send(path).ok();
							tx.send(parent).ok();
						}
						EventKind::Remove(_) => {
							tx.send(path).ok();
							tx.send(parent).ok();
						}
						_ => (),
					}
				}
			},
			Default::default(),
		);

		let instance = Self { watcher: watcher.unwrap(), watched: Default::default() };
		tokio::spawn(Self::changed(rx, instance.watched.clone()));
		instance
	}

	pub(super) fn watch(&mut self, mut watched: BTreeSet<&Url>) {
		let (to_unwatch, to_watch): (BTreeSet<_>, BTreeSet<_>) = {
			let guard = self.watched.read();
			let keys = guard.keys().collect::<BTreeSet<_>>();
			(
				keys.difference(&watched).map(|&x| x.clone()).collect(),
				watched.difference(&keys).map(|&x| x.clone()).collect(),
			)
		};

		for u in to_unwatch {
			self.watcher.unwatch(&u).ok();
		}
		for u in to_watch {
			if self.watcher.watch(&u, RecursiveMode::NonRecursive).is_err() {
				watched.remove(&u);
			}
		}

		let mut to_resolve = Vec::new();
		let mut guard = self.watched.write();
		*guard = watched
			.into_iter()
			.map(|k| {
				if let Some((k, v)) = guard.remove_entry(k) {
					(k, v)
				} else {
					to_resolve.push(k.clone());
					(k.clone(), None)
				}
			})
			.collect();
		guard.sort_unstable_by(|_, a, _, b| b.cmp(a));

		let lock = self.watched.clone();
		tokio::spawn(async move {
			let mut ext = IndexMap::new();
			for k in to_resolve {
				match fs::canonicalize(&k).await {
					Ok(v) if v != *k => {
						ext.insert(k, Some(Url::from(v)));
					}
					_ => {}
				}
			}

			let mut guard = lock.write();
			guard.extend(ext);
			guard.sort_unstable_by(|_, a, _, b| b.cmp(a));
		});
	}

	pub(super) fn trigger_dirs(&self, dirs: &[&Url]) {
		let watched = self.watched.clone();
		let dirs: Vec<_> = dirs.iter().map(|&u| u.clone()).collect();
		tokio::spawn(async move {
			for dir in dirs {
				Self::dir_changed(&dir, watched.clone()).await;
			}
		});
	}

	async fn changed(
		mut rx: StreamBuf<UnboundedReceiverStream<Url>>,
		watched: Arc<RwLock<IndexMap<Url, Option<Url>>>>,
	) {
		while let Some(paths) = rx.next().await {
			let (mut files, mut dirs): (Vec<_>, Vec<_>) = Default::default();
			for path in paths.into_iter().collect::<BTreeSet<_>>() {
				if fs::symlink_metadata(&path).await.map(|m| !m.is_dir()).unwrap_or(false) {
					files.push(path);
				} else {
					dirs.push(path);
				}
			}

			Self::file_changed(files.iter().collect()).await;
			for file in files {
				emit!(Files(FilesOp::IOErr(file)));
			}

			for dir in dirs {
				Self::dir_changed(&dir, watched.clone()).await;
			}
		}
	}

	async fn file_changed(urls: Vec<&Url>) {
		if let Ok(mimes) = external::file(&urls).await {
			emit!(Mimetype(mimes));
		}
	}

	async fn dir_changed(url: &Url, watched: Arc<RwLock<IndexMap<Url, Option<Url>>>>) {
		let linked = watched
			.read()
			.iter()
			.map_while(|(k, v)| v.as_ref().and_then(|v| url.strip_prefix(v)).map(|v| k.__join(v)))
			.collect::<Vec<_>>();

		let result = Files::read_dir(url).await;
		if linked.is_empty() {
			emit!(Files(match result {
				Ok(items) => FilesOp::Read(url.clone(), items),
				Err(_) => FilesOp::IOErr(url.clone()),
			}));
			return;
		}

		for ori in linked {
			emit!(Files(match &result {
				Ok(items) => {
					let mut files = Vec::with_capacity(items.len());
					for item in items {
						let mut file = item.clone();
						file.set_url(ori.__join(item.url().strip_prefix(url).unwrap()));
						files.push(file);
					}
					FilesOp::Read(ori, files)
				}
				Err(_) => FilesOp::IOErr(ori),
			}));
		}
	}
}
