use std::{path::Path, time::Duration};

use anyhow::Result;
use hashbrown::HashSet;
use notify::{PollWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::{pin, sync::mpsc::{self, UnboundedReceiver}};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tracing::error;
use yazi_fs::{File, FilesOp, provider};
use yazi_shared::url::{UrlBuf, UrlLike};
use yazi_vfs::VfsFile;

use crate::{Reporter, WATCHER};

pub(crate) struct Local(Box<dyn notify::Watcher + Send>);

impl Local {
	pub(crate) fn serve(rx: mpsc::UnboundedReceiver<UrlBuf>, reporter: Reporter) -> Self {
		tokio::spawn(Self::changed(rx));

		let config = notify::Config::default().with_poll_interval(Duration::from_millis(500));
		let handler = move |res: Result<notify::Event, notify::Error>| {
			if let Ok(event) = res
				&& !event.kind.is_access()
			{
				reporter.report(event.paths);
			}
		};

		if cfg!(target_os = "netbsd") || yazi_adapter::WSL.get() {
			return Self(Box::new(PollWatcher::new(handler, config).unwrap()));
		}

		Self(match RecommendedWatcher::new(handler.clone(), config) {
			Ok(watcher) => Box::new(watcher),
			Err(e) => {
				error!("Falling back to PollWatcher due to RecommendedWatcher init failure: {e:?}");
				Box::new(PollWatcher::new(handler, config).unwrap())
			}
		})
	}

	pub(crate) fn watch(&mut self, path: &Path) -> Result<()> {
		match self.0.watch(path, RecursiveMode::NonRecursive) {
			Ok(()) => Ok(()),
			Err(e) if matches!(e.kind, notify::ErrorKind::PathNotFound) => Ok(()),
			Err(e) => Err(e)?,
		}
	}

	pub(crate) fn unwatch(&mut self, path: &Path) -> Result<()> {
		match self.0.unwatch(path) {
			Ok(()) => Ok(()),
			Err(e) if matches!(e.kind, notify::ErrorKind::WatchNotFound) => Ok(()),
			Err(e) => Err(e)?,
		}
	}

	async fn changed(rx: UnboundedReceiver<UrlBuf>) {
		// TODO: revert this once a new notification is implemented
		let rx = UnboundedReceiverStream::new(rx).chunks_timeout(1000, Duration::from_millis(250));
		pin!(rx);

		while let Some(chunk) = rx.next().await {
			let urls: HashSet<_> = chunk.into_iter().collect();

			let _permit = WATCHER.acquire().await.unwrap();
			let mut ops = Vec::with_capacity(urls.len());

			for u in urls {
				let Some((parent, urn)) = u.pair() else { continue };
				let Ok(file) = File::new(&u).await else {
					ops.push(FilesOp::Deleting(parent.into(), [urn.into()].into()));
					continue;
				};

				if let Some(p) = file.url.as_local()
					&& !provider::local::match_name_case(p).await
				{
					ops.push(FilesOp::Deleting(parent.into(), [urn.into()].into()));
					continue;
				}

				ops.push(FilesOp::Upserting(parent.into(), [(urn.into(), file)].into()));
			}

			FilesOp::mutate(ops);
		}
	}
}
