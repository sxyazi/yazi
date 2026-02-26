use futures::{StreamExt, stream};
use hashbrown::HashSet;
use tracing::error;
use yazi_fs::FsUrl;
use yazi_shared::{LastValue, url::{UrlBuf, UrlCow, UrlLike}};

use crate::{Reporter, WATCHED, Watchee, backend::Backend};

pub struct Watcher {
	last:     LastValue<HashSet<UrlBuf>>,
	reporter: Reporter,
}

impl Watcher {
	pub fn serve() -> Self {
		let last = LastValue::default();

		let backend = Backend::serve();
		let reporter = backend.reporter.clone();

		tokio::spawn(Self::watched(last.clone(), backend));
		Self { last, reporter }
	}

	pub fn watch<'a, I>(&mut self, urls: I)
	where
		I: IntoIterator,
		I::Item: Into<UrlCow<'a>>,
	{
		let it = urls.into_iter();
		let mut urls = HashSet::with_capacity(it.size_hint().0);

		for url in it.map(Into::into) {
			if !url.is_absolute() {
				continue;
			} else if let Some(cache) = url.cache() {
				urls.insert(cache.into());
			}
			urls.insert(url.into_owned());
		}

		self.last.set(urls);
	}

	pub fn report<'a, I>(&self, urls: I)
	where
		I: IntoIterator,
		I::Item: Into<UrlCow<'a>>,
	{
		self.reporter.report(urls);
	}

	async fn watched(last: LastValue<HashSet<UrlBuf>>, mut backend: Backend) {
		loop {
			let (to_unwatch, to_watch) = Self::diff(last.get().await).await;

			if !to_unwatch.is_empty() || !to_watch.is_empty() {
				backend = Self::sync(backend, to_unwatch, to_watch).await;
				backend = backend.sync().await;
			}
		}
	}

	async fn diff(urls: HashSet<UrlBuf>) -> (Vec<Watchee<'static>>, HashSet<Watchee<'static>>) {
		let mut new: HashSet<_> = stream::iter(urls).then(Watchee::new).collect().await;
		let old = WATCHED.read();

		let to_unwatch = old.difference(&new).map(Watchee::to_static).collect();
		new.retain(|watchee| !old.contains(watchee));

		(to_unwatch, new)
	}

	async fn sync(
		mut backend: Backend,
		to_unwatch: Vec<Watchee<'static>>,
		to_watch: HashSet<Watchee<'static>>,
	) -> Backend {
		tokio::task::spawn_blocking(move || {
			for watchee in to_unwatch {
				match backend.unwatch(&watchee) {
					Ok(()) => _ = WATCHED.write().remove(&watchee),
					Err(e) => error!("Unwatch failed: {e:?}"),
				}
			}
			for mut watchee in to_watch {
				match backend.watch(&mut watchee) {
					Ok(()) => _ = WATCHED.write().insert(watchee),
					Err(e) if matches!(e.kind, notify::ErrorKind::PathNotFound) => {}
					Err(e) => error!("Watch failed: {e:?}"),
				}
			}
			backend
		})
		.await
		.unwrap()
	}
}
