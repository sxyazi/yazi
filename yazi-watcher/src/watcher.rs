use std::time::Duration;

use hashbrown::HashSet;
use tokio::{pin, sync::{mpsc::{self, UnboundedReceiver}, watch}};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_fs::{File, FilesOp, provider::local};
use yazi_shared::url::UrlBuf;

use crate::{LINKED, Linked, WATCHED, WATCHER, backend::Backend};

pub struct Watcher {
	in_tx:  watch::Sender<HashSet<UrlBuf>>,
	out_tx: mpsc::UnboundedSender<UrlBuf>,
}

impl Watcher {
	pub fn serve() -> Self {
		let (in_tx, in_rx) = watch::channel(Default::default());
		let (out_tx, out_rx) = mpsc::unbounded_channel();

		let backend = Backend::serve(out_tx.clone());

		tokio::spawn(Self::fan_in(in_rx, backend));
		tokio::spawn(Self::fan_out(out_rx));

		Self { in_tx, out_tx }
	}

	pub fn watch<'a>(&mut self, it: impl Iterator<Item = &'a UrlBuf>) {
		self.in_tx.send(it.cloned().collect()).ok();
	}

	pub fn push_files(&self, urls: Vec<UrlBuf>) { Backend::push_files(&self.out_tx, urls); }

	async fn fan_in(mut rx: watch::Receiver<HashSet<UrlBuf>>, mut backend: Backend) {
		loop {
			let (to_unwatch, to_watch) = WATCHED.read().diff(&rx.borrow_and_update());
			backend = backend.sync(to_unwatch, to_watch).await;

			if !rx.has_changed().unwrap_or(false) {
				Linked::sync(&LINKED, &WATCHED).await;
			}

			if rx.changed().await.is_err() {
				break;
			}
		}
	}

	async fn fan_out(rx: UnboundedReceiver<UrlBuf>) {
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

				if let Some(p) = file.url.as_path()
					&& !local::must_case_match(p).await
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
