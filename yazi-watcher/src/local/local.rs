use std::{path::Path, time::Duration};

use hashbrown::HashSet;
use notify::{PollWatcher, RecommendedWatcher, RecursiveMode, Result, Watcher};
use tokio::{pin, sync::mpsc::{self, UnboundedReceiver}};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tracing::error;
use yazi_fs::{File, FilesOp, mounts::PARTITIONS, provider::{self, Provider}};
use yazi_shared::url::{UrlBuf, UrlLike};
use yazi_vfs::VfsFile;

use crate::{Reporter, WATCHER, Watchee};

pub(crate) struct Local {
	primary:     Option<RecommendedWatcher>,
	alternative: PollWatcher,
}

impl Local {
	pub(crate) fn serve(rx: mpsc::UnboundedReceiver<UrlBuf>, reporter: Reporter) -> Self {
		tokio::spawn(Self::changed(rx));

		let config = notify::Config::default().with_poll_interval(Duration::from_secs(1));
		let handler = move |res: Result<notify::Event>| {
			if let Ok(event) = res
				&& !event.kind.is_access()
			{
				reporter.report(event.paths);
			}
		};

		let primary = RecommendedWatcher::new(handler.clone(), config);
		let alternative = PollWatcher::new(handler, config).unwrap();

		if let Err(e) = &primary {
			error!("Failed to initialize primary watcher: {e:?}");
		}

		Self { primary: primary.ok(), alternative }
	}

	pub(crate) fn watch(&mut self, watchee: &mut Watchee) -> Result<()> {
		let (path, alt) =
			watchee.as_local_mut().ok_or_else(|| notify::Error::generic("Not a local watchee"))?;

		if let Some(primary) = self.primary.as_mut().filter(|_| !*alt) {
			match primary.watch(path, RecursiveMode::NonRecursive) {
				Ok(()) => return Ok(()),
				Err(e) => tracing::warn!("Failed to watch {path:?} with primary watcher: {e:?}"),
			}
		}

		tracing::debug!("Watching {path:?} with alternative watcher");
		*alt = true;
		self.alternative.watch(path, RecursiveMode::NonRecursive)
	}

	pub(crate) fn unwatch(&mut self, watchee: &Watchee) -> Result<()> {
		let (path, alt) =
			watchee.as_local().ok_or_else(|| notify::Error::generic("Not a local watchee"))?;

		let result = if alt {
			self.alternative.unwatch(path)
		} else if let Some(primary) = &mut self.primary {
			primary.unwatch(path)
		} else {
			Ok(())
		};

		match result {
			Ok(()) => Ok(()),
			Err(e) if matches!(e.kind, notify::ErrorKind::WatchNotFound) => Ok(()),
			Err(e) => Err(e)?,
		}
	}

	pub(crate) async fn soundless(path: &Path) -> bool {
		if cfg!(target_os = "netbsd") || yazi_adapter::WSL.get() {
			return true;
		}

		match provider::local::Local::regular(path).metadata().await {
			Ok(cha) => PARTITIONS.read().soundless(cha),
			Err(_) => true,
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
