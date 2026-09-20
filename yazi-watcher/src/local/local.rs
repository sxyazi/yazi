use std::{io::ErrorKind, path::Path, time::Duration};

use hashbrown::HashSet;
use notify::{PollWatcher, RecommendedWatcher, RecursiveMode, Result, Watcher};
use tokio::{pin, sync::mpsc::{self, UnboundedReceiver}};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_fs::{casefold::Casefold, engine::{self, Engine}, mounts::PARTITIONS, op::FilesOp};
use yazi_macro::error;
use yazi_shared::url::{UrlBuf, UrlLike};
use yazi_vfs::maybe_exists;

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
				Err(e) => yazi_macro::warn!("Failed to watch {path:?} with primary watcher: {e:?}"),
			}
		}

		yazi_macro::debug!("Watching {path:?} with alternative watcher");
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

		match engine::local::Local::regular(path).metadata().await {
			Ok(stat) => PARTITIONS.read().soundless(stat),
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

			for url in urls {
				let Some(path) = url.as_local() else { continue };
				let Some((trail, key)) = url.pair() else { continue };

				let file = match engine::local::Local::regular(path).file().await {
					Ok(file) => file,
					Err(e) if e.kind() == ErrorKind::NotFound => {
						ops.push(FilesOp::Delete(trail.into(), [key.into()].into()));
						continue;
					}
					Err(e) => {
						error!("Failed to update {url}: {e:?}");
						continue;
					}
				};

				match Casefold::match_name_case(path).await {
					Ok(true) => {
						ops.push(FilesOp::Upsert(trail.into(), [(key.into(), file)].into()));
					}
					Ok(false) => {
						ops.push(FilesOp::Delete(trail.into(), [key.into()].into()));
					}
					Err(e) if e.kind() == ErrorKind::NotFound && !maybe_exists(&url).await => {
						ops.push(FilesOp::Delete(trail.into(), [key.into()].into()));
					}
					Err(e) => {
						error!("Failed to match filename case for {url}: {e:?}");
					}
				};
			}

			FilesOp::mutate(ops);
		}
	}
}
