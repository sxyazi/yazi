use hashbrown::HashSet;
use tokio::sync::watch;
use tracing::error;
use yazi_fs::FsUrl;
use yazi_shared::url::{UrlBuf, UrlCow, UrlLike};

use crate::{Reporter, WATCHED, backend::Backend};

pub struct Watcher {
	tx:       watch::Sender<HashSet<UrlBuf>>,
	reporter: Reporter,
}

impl Watcher {
	pub fn serve() -> Self {
		let (tx, rx) = watch::channel(Default::default());

		let backend = Backend::serve();
		let reporter = backend.reporter.clone();

		tokio::spawn(Self::watched(rx, backend));
		Self { tx, reporter }
	}

	pub fn watch<'a, I>(&mut self, urls: I)
	where
		I: IntoIterator,
		I::Item: Into<UrlCow<'a>>,
	{
		let it = urls.into_iter();
		let mut set = HashSet::with_capacity(it.size_hint().0);

		for url in it.map(Into::into) {
			if !url.is_absolute() {
				continue;
			} else if let Some(cache) = url.cache() {
				set.insert(cache.into());
			}
			set.insert(url.into_owned());
		}

		self.tx.send(set).ok();
	}

	pub fn report<'a, I>(&self, urls: I)
	where
		I: IntoIterator,
		I::Item: Into<UrlCow<'a>>,
	{
		self.reporter.report(urls);
	}

	async fn watched(mut rx: watch::Receiver<HashSet<UrlBuf>>, mut backend: Backend) {
		loop {
			let (to_unwatch, to_watch) = WATCHED.read().diff(&rx.borrow_and_update());

			if !to_unwatch.is_empty() || !to_watch.is_empty() {
				backend = Self::sync(backend, to_unwatch, to_watch).await;
				backend = backend.sync().await;
			}

			if rx.changed().await.is_err() {
				break;
			}
		}
	}

	async fn sync(mut backend: Backend, to_unwatch: Vec<UrlBuf>, to_watch: Vec<UrlBuf>) -> Backend {
		tokio::task::spawn_blocking(move || {
			for u in to_unwatch {
				match backend.unwatch(&u) {
					Ok(()) => WATCHED.write().remove(&u),
					Err(e) => error!("Unwatch failed: {e:?}"),
				}
			}
			for u in to_watch {
				match backend.watch(&u) {
					Ok(()) => WATCHED.write().insert(u),
					Err(e) => error!("Watch failed: {e:?}"),
				}
			}
			backend
		})
		.await
		.unwrap()
	}
}
