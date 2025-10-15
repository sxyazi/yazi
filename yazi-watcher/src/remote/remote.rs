use std::time::{Duration, SystemTime};

use anyhow::Result;
use hashbrown::HashSet;
use tokio::{pin, sync::mpsc::UnboundedReceiver};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_fs::{File, FilesOp};
use yazi_proxy::MgrProxy;
use yazi_shared::url::{Url, UrlBuf, UrlLike};
use yazi_vfs::VfsFile;

use crate::{Reporter, WATCHER};

pub(crate) struct Remote;

impl Remote {
	pub(crate) fn serve(rx: UnboundedReceiver<UrlBuf>, _reporter: Reporter) -> Self {
		tokio::spawn(Self::changed(rx));

		Self
	}

	pub(crate) fn watch(&mut self, _url: Url) -> Result<()> { Ok(()) }

	pub(crate) fn unwatch(&mut self, _url: Url) -> Result<()> { Ok(()) }

	async fn changed(rx: UnboundedReceiver<UrlBuf>) {
		let rx = UnboundedReceiverStream::new(rx).chunks_timeout(1000, Duration::from_millis(250));
		pin!(rx);

		while let Some(chunk) = rx.next().await {
			let urls: HashSet<_> = chunk.into_iter().collect();

			let _permit = WATCHER.acquire().await.unwrap();

			let mut ops = Vec::with_capacity(urls.len());
			let mut ups = Vec::with_capacity(urls.len());

			for u in urls {
				let Some((parent, urn)) = u.pair() else { continue };
				let Ok(mut file) = File::new(&u).await else {
					ops.push(FilesOp::Deleting(parent.into(), [urn.into()].into()));
					continue;
				};

				let is_file = file.is_file();
				file.cha.ctime = Some(SystemTime::now());

				ops.push(FilesOp::Upserting(parent.into(), [(urn.into(), file)].into()));
				if is_file {
					ups.push(u);
				}
			}

			FilesOp::mutate(ops);
			MgrProxy::upload(ups);
		}
	}
}
