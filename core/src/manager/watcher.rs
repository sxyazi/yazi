use std::{collections::BTreeSet, sync::Arc, time::Duration};

use indexmap::IndexMap;
use notify::{event::{MetadataKind, ModifyKind}, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _Watcher};
use parking_lot::RwLock;
use shared::Url;
use tokio::{fs, pin, sync::mpsc::{self, UnboundedReceiver}};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use crate::{emit, external, files::{File, Files, FilesOp}};

pub struct Watcher {
	watcher: RecommendedWatcher,
	watched: Arc<RwLock<IndexMap<Url, Option<Url>>>>,
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
		watched.retain(|&u| u.is_regular());
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
		let dirs: Vec<_> = dirs.iter().filter(|&u| u.is_regular()).map(|&u| u.clone()).collect();
		if dirs.is_empty() {
			return;
		}

		let watched = self.watched.clone();
		tokio::spawn(async move {
			for dir in dirs {
				Self::dir_changed(&dir, watched.clone()).await;
			}
		});
	}

	async fn changed(rx: UnboundedReceiver<Url>, watched: Arc<RwLock<IndexMap<Url, Option<Url>>>>) {
		let rx = UnboundedReceiverStream::new(rx).chunks_timeout(100, Duration::from_millis(200));
		pin!(rx);

		while let Some(paths) = rx.next().await {
			let (mut files, mut dirs): (Vec<_>, Vec<_>) = Default::default();
			for path in paths.into_iter().collect::<BTreeSet<_>>() {
				if fs::symlink_metadata(&path).await.map(|m| !m.is_dir()).unwrap_or(false) {
					files.push(path);
				} else {
					dirs.push(path);
				}
			}

			Self::file_changed(&files, watched.clone()).await;
			for file in files {
				emit!(Files(FilesOp::IOErr(file)));
			}

			for dir in dirs {
				Self::dir_changed(&dir, watched.clone()).await;
			}
		}
	}

	async fn file_changed(urls: &[Url], watched: Arc<RwLock<IndexMap<Url, Option<Url>>>>) {
		let Ok(mut mimes) = external::file(urls).await else {
			return;
		};

		let linked: Vec<_> = watched
			.read()
			.iter()
			.filter_map(|(k, v)| v.as_ref().map(|v| (k, v)))
			.fold(Vec::new(), |mut aac, (k, v)| {
				mimes
					.iter()
					.filter(|(f, _)| f.parent().map(|p| p == **v) == Some(true))
					.for_each(|(f, m)| aac.push((k.join(f.file_name().unwrap()), m.clone())));
				aac
			});

		mimes.extend(linked);
		emit!(Mimetype(mimes));
	}

	async fn dir_changed(url: &Url, watched: Arc<RwLock<IndexMap<Url, Option<Url>>>>) {
		let linked: Vec<_> = watched
			.read()
			.iter()
			.filter_map(|(k, v)| v.as_ref().map(|v| (k, v)))
			.filter(|(_, v)| *v == url)
			.map(|(k, _)| k.clone())
			.collect();

		let Ok(rx) = Files::from_dir(url).await else {
			emit!(Files(FilesOp::IOErr(url.clone())));
			for ori in linked {
				emit!(Files(FilesOp::IOErr(ori)));
			}
			return;
		};

		let linked_files = |files: &[File], ori: &Url| -> Vec<File> {
			let mut new = Vec::with_capacity(files.len());
			for file in files {
				let mut file = file.clone();
				file.set_url(ori.join(file.url().strip_prefix(url).unwrap()));
				new.push(file);
			}
			new
		};

		let files: Vec<_> = UnboundedReceiverStream::new(rx).collect().await;
		for ori in linked {
			let files = linked_files(&files, &ori);
			emit!(Files(FilesOp::Full(ori, files)));
		}
		emit!(Files(FilesOp::Full(url.clone(), files)));
	}
}
