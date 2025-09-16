use anyhow::Result;
use hashbrown::HashSet;
use tokio::sync::mpsc;
use tracing::error;
use yazi_shared::url::{Url, UrlBuf};

use crate::{LINKED, WATCHED, backend};

pub(crate) struct Backend {
	local: backend::Local,
}

impl Backend {
	pub(crate) fn serve(out_tx: mpsc::UnboundedSender<UrlBuf>) -> Self {
		#[cfg(any(target_os = "linux", target_os = "macos"))]
		yazi_fs::mounts::Partitions::monitor(&yazi_fs::mounts::PARTITIONS, || {
			yazi_macro::err!(yazi_dds::Pubsub::pub_after_mount())
		});

		Self { local: backend::Local::serve(out_tx) }
	}

	pub(crate) async fn sync(mut self, to_unwatch: Vec<UrlBuf>, to_watch: Vec<UrlBuf>) -> Self {
		if to_unwatch.is_empty() && to_watch.is_empty() {
			return self;
		}

		tokio::task::spawn_blocking(move || {
			for u in to_unwatch {
				match self.unwatch(&u) {
					Ok(()) => WATCHED.write().remove(&u),
					Err(e) => error!("Unwatch failed: {e:?}"),
				}
			}
			for u in to_watch {
				match self.watch(&u) {
					Ok(()) => WATCHED.write().insert(u),
					Err(e) => error!("Watch failed: {e:?}"),
				}
			}
			self
		})
		.await
		.unwrap()
	}

	pub(crate) fn push_files<I, T>(out_tx: &mpsc::UnboundedSender<UrlBuf>, urls: I)
	where
		I: IntoIterator<Item = T>,
		T: Into<UrlBuf>,
	{
		let (mut todo, watched) = (HashSet::new(), WATCHED.read());
		for url in urls.into_iter().map(Into::into) {
			let Some(parent) = url.parent() else { continue };
			if todo.contains(&parent) {
				todo.insert(url);
			} else if watched.contains(parent)
				|| LINKED.read().from_dir(parent).any(|p| watched.contains(Url::regular(p)))
			{
				todo.insert(parent.to_owned());
				todo.insert(url);
			}
		}
		todo.into_iter().for_each(|u| _ = out_tx.send(u));
	}

	fn watch<'a>(&mut self, url: impl Into<Url<'a>>) -> Result<()> {
		if let Some(path) = url.into().as_path() { self.local.watch(path) } else { Ok(()) }
	}

	fn unwatch<'a>(&mut self, url: impl Into<Url<'a>>) -> Result<()> {
		if let Some(path) = url.into().as_path() { self.local.unwatch(path) } else { Ok(()) }
	}
}
