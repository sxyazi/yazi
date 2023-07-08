use std::{collections::BTreeSet, path::{Path, PathBuf}};

use notify::{RecommendedWatcher, Watcher as _Watcher};
use tokio::sync::mpsc::{self, Sender};

use super::Folder;
use crate::emit;

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
					match event.kind {
						notify::EventKind::Create(_) => {}
						notify::EventKind::Modify(_) => {}
						notify::EventKind::Remove(_) => {}
						_ => return,
					}

					let path = if event.paths.len() > 0 {
						event.paths[0].parent().unwrap_or(&event.paths[0])
					} else {
						return;
					};

					tx.blocking_send(path.to_path_buf()).ok();
				}
			},
			notify::Config::default(),
		)
		.unwrap();

		tokio::spawn(async move {
			while let Some(path) = rx.recv().await {
				Folder::read(&path).await;
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
