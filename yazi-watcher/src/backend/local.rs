use std::{path::Path, time::Duration};

use anyhow::Result;
use notify::{ErrorKind::WatchNotFound, PollWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use yazi_shared::url::UrlBuf;

use crate::backend::Backend;

pub(super) struct Local(Box<dyn notify::Watcher + Send>);

impl Local {
	pub(super) fn serve(out_tx: mpsc::UnboundedSender<UrlBuf>) -> Self {
		let handler = move |res: Result<notify::Event, notify::Error>| {
			let Ok(event) = res else { return };
			if event.kind.is_access() {
				return;
			}
			Backend::push_files(&out_tx, event.paths);
		};

		let config = notify::Config::default().with_poll_interval(Duration::from_millis(500));
		Self(if yazi_adapter::WSL.get() || cfg!(target_os = "netbsd") {
			Box::new(PollWatcher::new(handler, config).unwrap())
		} else {
			Box::new(RecommendedWatcher::new(handler, config).unwrap())
		})
	}

	pub(super) fn watch(&mut self, path: &Path) -> Result<()> {
		Ok(self.0.watch(path, RecursiveMode::NonRecursive)?)
	}

	pub(super) fn unwatch(&mut self, path: &Path) -> Result<()> {
		match self.0.unwatch(path) {
			Ok(()) => Ok(()),
			Err(e) if matches!(e.kind, WatchNotFound) => Ok(()),
			Err(e) => Err(e)?,
		}
	}
}
