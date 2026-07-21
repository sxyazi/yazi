use futures::{StreamExt, stream};
use indexmap::IndexSet;
use tracing::error;
use yazi_fs::{FsUrl, file::File};
use yazi_shared::{LastValue, url::{UrlBuf, UrlCow, UrlLike}};

use crate::{Refresher, Reporter, WATCHED, Watchee, backend::Backend};

pub struct Watcher {
	last:          LastValue<IndexSet<UrlBuf>>,
	reporter:      Reporter,
	pub refresher: Refresher,
}

impl Watcher {
	pub fn serve() -> Self {
		let last = LastValue::default();
		let refresher = Refresher::serve();

		let backend = Backend::serve();
		let reporter = backend.reporter.clone();

		tokio::spawn(Self::run(last.clone(), backend));
		Self { last, reporter, refresher }
	}

	pub fn watch<I>(&mut self, files: I)
	where
		I: IntoIterator,
		I::Item: Into<File>,
	{
		let it = files.into_iter();
		let mut urls = IndexSet::with_capacity(it.size_hint().0);
		let mut files = IndexSet::with_capacity(it.size_hint().0);

		for file in it.map(Into::into) {
			if !file.url.is_absolute() {
				continue;
			} else if let Some(cache) = file.url.cache() {
				urls.insert(cache.into());
			}
			urls.insert(file.url.clone());
			files.insert(file.into());
		}

		self.last.set(urls);
		self.refresher.sync(files);
	}

	pub fn report<'a, I>(&self, urls: I)
	where
		I: IntoIterator,
		I::Item: Into<UrlCow<'a>>,
	{
		self.reporter.report(urls);
	}

	async fn run(last: LastValue<IndexSet<UrlBuf>>, mut backend: Backend) {
		loop {
			let (to_unwatch, to_watch) = Self::diff(last.get().await).await;

			if !to_unwatch.is_empty() || !to_watch.is_empty() {
				backend = Self::sync(backend, to_unwatch, to_watch).await;
				backend = backend.sync().await;
			}
		}
	}

	async fn diff(urls: IndexSet<UrlBuf>) -> (Vec<Watchee<'static>>, IndexSet<Watchee<'static>>) {
		let mut new: IndexSet<_> = stream::iter(urls).then(Watchee::new).collect().await;
		let old = WATCHED.read();

		let to_unwatch = old.difference(&new).map(Watchee::to_static).collect();
		new.retain(|watchee| !old.contains(watchee));

		(to_unwatch, new)
	}

	async fn sync(
		mut backend: Backend,
		to_unwatch: Vec<Watchee<'static>>,
		to_watch: IndexSet<Watchee<'static>>,
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
					Ok(()) => WATCHED.write().insert(watchee),
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
