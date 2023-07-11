use std::{collections::BTreeSet, path::{Path, PathBuf}};

use notify::{event::{MetadataKind, ModifyKind}, EventKind, RecommendedWatcher, Watcher as _Watcher};
use tokio::sync::mpsc::{self, Sender};

use crate::{core::files::{Files, FilesOp}, emit};

pub struct Watcher {
	tx: Sender<PathBuf>,

	watcher: RecommendedWatcher,
	watched: BTreeSet<PathBuf>,
}

impl Watcher {
	pub fn init() -> Self {
		let (watcher, tx) = Self::start();

		Self { tx, watcher, watched: Default::default() }
	}

	fn start() -> (RecommendedWatcher, Sender<PathBuf>) {
		let (tx, mut rx) = mpsc::channel(50);

		let watcher = RecommendedWatcher::new(
			{
				let tx = tx.clone();
				move |res: Result<notify::Event, notify::Error>| {
					if res.is_err() {
						return;
					}

					let event = res.unwrap();
					let path = if let Some(first) = event.paths.first() {
						first.clone()
					} else {
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
						_ => return,
					}
				}
			},
			notify::Config::default(),
		)
		.unwrap();

		tokio::spawn(async move {
			while let Some(path) = rx.recv().await {
				emit!(Files(match Files::read_dir(&path).await {
					Ok(items) => FilesOp::Read(path, items),
					Err(_) => FilesOp::IOErr(path),
				}));
			}
		});

		(watcher, tx)
	}

	pub(super) fn watch(&mut self, to_watch: BTreeSet<PathBuf>) {
		for p in to_watch.difference(&self.watched) {
			self.watcher.watch(&p, notify::RecursiveMode::NonRecursive).ok();
		}
		for p in self.watched.difference(&to_watch) {
			self.watcher.unwatch(p).ok();
		}
		self.watched = to_watch;
	}

	pub(super) fn trigger(&self, path: &Path) {
		let tx = self.tx.clone();
		let path = path.to_path_buf();
		tokio::spawn(async move {
			tx.send(path).await.ok();
		});
	}
}
